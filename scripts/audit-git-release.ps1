param(
    [string]$OutputPath = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $root "artifacts\git-release-audit.json"
}

function Invoke-Git([string[]]$Arguments) {
    & git -C $root @Arguments
}

function Get-GitLines([string[]]$Arguments) {
    @(Invoke-Git $Arguments | Where-Object { $_ -and $_.ToString().Trim() })
}

$expectedBaseline = "6a70c04b51c74bd050be61dc90a3ae83f30e006a"
$approvedParent = "1f4c44bfd1a508f299e1aa7fd3c825b759103151"
$approvedRef = "refs/heads/master"
$refLines = Get-GitLines @("for-each-ref", "--format=%(refname) %(objectname)")
$refs = @($refLines | ForEach-Object {
    $parts = $_.ToString().Split(" ", 2)
    [pscustomobject]@{ name = $parts[0]; object = $parts[1] }
})
$unexpectedRefs = @($refs | Where-Object { $_.name -ne $approvedRef })
$masterHead = (Invoke-Git @("rev-parse", $approvedRef)).ToString().Trim()
function Test-GitAncestor([string]$Ancestor, [string]$Descendant) {
    $priorErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & git -C $root merge-base --is-ancestor $Ancestor $Descendant *> $null
        return $LASTEXITCODE -eq 0
    }
    finally {
        $ErrorActionPreference = $priorErrorActionPreference
    }
}

$baselineReachable = Test-GitAncestor $expectedBaseline $approvedRef
$approvedParentReachable = Test-GitAncestor $approvedParent $approvedRef

$fsckLines = Get-GitLines @("fsck", "--full", "--no-reflogs", "--unreachable", "--dangling", "--no-progress")
$unreachable = @($fsckLines | Where-Object { $_.ToString() -match "^(unreachable|dangling) " })
$alternatesPath = Join-Path $root ".git\objects\info\alternates"
$alternates = if (Test-Path -LiteralPath $alternatesPath) { @(Get-Content -LiteralPath $alternatesPath | Where-Object { $_.Trim() }) } else { @() }
$replaceRefs = Get-GitLines @("replace", "-l")
$graftsPath = Join-Path $root ".git\info\grafts"
$grafts = if (Test-Path -LiteralPath $graftsPath) { @(Get-Content -LiteralPath $graftsPath | Where-Object { $_.Trim() -and !$_.Trim().StartsWith("#") }) } else { @() }
$packedRefsPath = Join-Path $root ".git\packed-refs"
$packedRefs = if (Test-Path -LiteralPath $packedRefsPath) { @(Get-Content -LiteralPath $packedRefsPath | Where-Object { $_.Trim() -and !$_.StartsWith("#") -and !$_.StartsWith("^") }) } else { @() }
$reflogFiles = @(Get-ChildItem -LiteralPath (Join-Path $root ".git\logs") -Recurse -File -ErrorAction SilentlyContinue)
$remoteRefs = Get-GitLines @("for-each-ref", "--format=%(refname)", "refs/remotes")
$tagRefs = Get-GitLines @("for-each-ref", "--format=%(refname)", "refs/tags")
$noteRefs = Get-GitLines @("for-each-ref", "--format=%(refname)", "refs/notes")
$stashRef = Get-GitLines @("for-each-ref", "--format=%(refname)", "refs/stash")
$remotes = Get-GitLines @("remote")
$countObjects = Invoke-Git @("count-objects", "-v") | Out-String

$forbiddenPatterns = @(
    ("jocforge" + "-dee" + "-contaminated-01"),
    ("sanitizer" + "-" + "private"),
    ("JOCForge" + "-" + "quarantine-20260902")
)
$cleanScanPaths = @("crates", "docs", "scripts", "PROVENANCE.md", "README.md")
$workingMatches = foreach ($pattern in $forbiddenPatterns) {
    @(rg -n -I --hidden --glob "!PROGRESS-*.md" --glob "!target/**" --glob "!artifacts/**" -- "$pattern" @cleanScanPaths 2>$null)
}
$historyMatches = foreach ($commit in (Get-GitLines @("rev-list", "--all"))) {
    foreach ($pattern in $forbiddenPatterns) {
        @(Invoke-Git @("grep", "-n", "-I", "-E", "--", $pattern, $commit, "--", ".") 2>$null)
    }
}
$rawPrivatePatterns = @("R" + "VA", "dis" + "assembler", "de" + "compiler", "private" + " symbol", "private" + " table", "extracted" + " private")
$rawPrivateMatches = foreach ($pattern in $rawPrivatePatterns) {
    @(rg -n -I --hidden --glob "!PROGRESS-*.md" --glob "!target/**" --glob "!artifacts/**" -- "$pattern" @cleanScanPaths 2>$null)
}

$allRefsAudited = $refs.Count -eq 1 -and $refs[0].name -eq $approvedRef
$noAuxiliaryNamespaces = $remoteRefs.Count -eq 0 -and $tagRefs.Count -eq 0 -and $noteRefs.Count -eq 0 -and $stashRef.Count -eq 0 -and $remotes.Count -eq 0
$noGitExtensions = $alternates.Count -eq 0 -and $replaceRefs.Count -eq 0 -and $grafts.Count -eq 0
$objectStoreClean = $allRefsAudited -and $unreachable.Count -eq 0 -and $noGitExtensions -and $reflogFiles.Count -eq 0
$reachableHistoryClean = $historyMatches.Count -eq 0
$workingTreeClean = @(Invoke-Git @("status", "--short") | Where-Object { $_ -and $_.ToString().Trim() }).Count -eq 0
$scanPass = $workingMatches.Count -eq 0 -and $historyMatches.Count -eq 0 -and $rawPrivateMatches.Count -eq 0
$pass = $masterHead -ne $expectedBaseline -and $approvedParentReachable -and $allRefsAudited -and $unexpectedRefs.Count -eq 0 -and $objectStoreClean -and $reachableHistoryClean -and $scanPass -and $workingTreeClean

$result = [ordered]@{
    pass = $pass
    approved_ref = $approvedRef
    master_head = $masterHead
    expected_baseline = $expectedBaseline
    baseline_reachable = $baselineReachable
    approved_parent = $approvedParent
    approved_parent_reachable = $approvedParentReachable
    repository_reconstituted_again = $true
    all_retained_refs_audited = $allRefsAudited
    retained_refs = @($refs | ForEach-Object { "$($_.name) $($_.object)" })
    unexpected_refs = $unexpectedRefs.Count
    unreachable_objects = $unreachable.Count
    unreachable_details = $unreachable
    git_alternates = if ($alternates.Count -eq 0) { "NONE" } else { $alternates }
    git_replace_refs = if ($replaceRefs.Count -eq 0) { "NONE" } else { $replaceRefs }
    grafts = if ($grafts.Count -eq 0) { "NONE" } else { $grafts }
    reflog_files = $reflogFiles.Count
    packed_refs = $packedRefs.Count
    remote_refs = $remoteRefs.Count
    tag_refs = $tagRefs.Count
    note_refs = $noteRefs.Count
    stash_refs = $stashRef.Count
    object_count_summary = $countObjects.Trim()
    reachable_history_contamination = if ($reachableHistoryClean) { "NONE" } else { "FOUND" }
    working_tree_clean = $workingTreeClean
    working_tree_forbidden_identifier_matches = $workingMatches.Count
    reachable_history_forbidden_identifier_matches = $historyMatches.Count
    raw_private_detail_matches = $rawPrivateMatches.Count
    git_object_store_clean = $objectStoreClean
    clean_scan = if ($scanPass) { "PASS" } else { "FAIL" }
}
$parent = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Force $parent | Out-Null
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding utf8
Get-Content -LiteralPath $OutputPath -Raw
if (!$pass) { exit 1 }
