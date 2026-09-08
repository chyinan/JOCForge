# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\spec-legal-zero"
$representatives = Join-Path $artifact "representatives"
New-Item -ItemType Directory -Force $representatives | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$pinnedOpenJoc = (Resolve-OpenJocValidator $root).executable
$openjocRoot = Join-Path (Split-Path -Parent $root) "OpenJOC"
$sourceOpenJoc = Join-Path $openjocRoot "target\release\openjoc.exe"
$pinIdentity = Assert-OpenJocValidatorPin $root $pinnedOpenJoc
$pinIdentity | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-identity.json") -Encoding utf8
$sourceRevision = (& git -C $openjocRoot rev-parse HEAD).Trim()

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

function Invoke-CheckedInDirectory([string]$directory, [string]$file, [string[]]$arguments) {
    Push-Location -LiteralPath $directory
    try {
        Invoke-Checked $file $arguments
    } finally {
        Pop-Location
    }
}

function Invoke-Captured([string]$file, [string[]]$arguments, [string]$log) {
    $previous = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & $file @arguments *> $log
    $exit = $LASTEXITCODE
    $ErrorActionPreference = $previous
    return $exit
}

function Get-Sha256([string]$path) {
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [IO.File]::ReadAllBytes($path)
        return ([BitConverter]::ToString($hasher.ComputeHash($bytes))).Replace("-", "")
    } finally {
        $hasher.Dispose()
    }
}

function Get-CellKey($row) {
    @(
        $row.profile,
        $row.container,
        $row.carriage,
        $row.emdf_profile,
        $row.partition,
        $row.dependent_count,
        $row.lfe_ownership,
        $row.bitrate_kbps
    ) -join "|"
}

$matrixCsv = Join-Path $root "artifacts\composition-capability-matrix.csv"
$matrixJson = Join-Path $root "artifacts\composition-capability-matrix.json"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--locked", "--", "composition-matrix", $matrixCsv, $matrixJson)
$matrixRows = Import-Csv -LiteralPath $matrixCsv
$currentKeys = @{}
$currentRowsByKey = @{}
foreach ($row in $matrixRows) {
    $key = Get-CellKey $row
    $currentKeys[$key] = $true
    $currentRowsByKey[$key] = $row
}
if ($matrixRows.Count -ne 140400 -or $currentKeys.Count -ne 140400) {
    throw "authoritative matrix row count or cell-key uniqueness changed"
}
$baselineHead = "3b88223353e161344609616d11ab0cd0dabcdd8c"
$baselineRoot = Join-Path ([IO.Path]::GetTempPath()) "jocforge-spec-legal-zero-baseline-$PID"
$baselineCsv = Join-Path $artifact "initial-baseline-regenerated.csv"
$baselineJson = Join-Path $artifact "initial-baseline-regenerated.json"
& git worktree add --detach $baselineRoot $baselineHead
if ($LASTEXITCODE -ne 0) { throw "failed to materialize the declared baseline commit" }
try {
    Invoke-CheckedInDirectory $baselineRoot "cargo" @("run", "-p", "jocforge", "--quiet", "--locked", "--", "composition-matrix", $baselineCsv, $baselineJson)
} finally {
    & git worktree remove --force $baselineRoot | Out-Null
}
$baselineRows = Import-Csv -LiteralPath $baselineCsv
$baselineGapCells = @($baselineRows | Where-Object verdict -eq "SPEC_LEGAL_NOT_IMPLEMENTED")
if ($baselineRows.Count -ne 140400 -or $baselineGapCells.Count -ne 18369) {
    throw "the declared baseline commit did not reproduce the recorded gap census"
}
$initialGapCells = Import-Csv -LiteralPath (Join-Path $artifact "initial-gap-cells.csv")
if ($initialGapCells.Count -ne 18369) { throw "baseline gap-cell artifact does not contain 18369 rows" }
$invalidInitialVerdicts = @($initialGapCells | Where-Object verdict -ne "SPEC_LEGAL_NOT_IMPLEMENTED")
if ($invalidInitialVerdicts.Count -ne 0) { throw "baseline gap-cell artifact contains a non-gap verdict" }
$baselineKeys = @{}
foreach ($row in $baselineGapCells) { $baselineKeys[(Get-CellKey $row)] = $row.reason }
$recordedKeys = @{}
foreach ($row in $initialGapCells) { $recordedKeys[(Get-CellKey $row)] = $row.reason }
if ($baselineKeys.Count -ne $recordedKeys.Count) { throw "baseline gap-cell artifact key count differs from the declared baseline" }
foreach ($key in $baselineKeys.Keys) {
    if (-not $recordedKeys.ContainsKey($key) -or $recordedKeys[$key] -ne $baselineKeys[$key]) {
        throw "baseline gap-cell artifact differs from the declared baseline at key $key"
    }
}
$missingBaselineCells = @($initialGapCells | Where-Object { -not $currentKeys.ContainsKey((Get-CellKey $_)) })
if ($missingBaselineCells.Count -ne 0) { throw "current matrix omitted baseline gap cells" }
$matrixCounts = [ordered]@{
    SUPPORTED_PASS = 0
    PUBLIC_AMBIGUOUS = 0
    NOT_APPLICABLE = 0
    SEMANTIC_CEILING = 0
    REJECTED_CAPACITY = 0
    INVALID = 0
    RESERVED = 0
    SPEC_LEGAL_NOT_IMPLEMENTED = 0
}
foreach ($group in ($matrixRows | Group-Object verdict)) { $matrixCounts[$group.Name] = $group.Count }
$matrixCounts | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $artifact "final-matrix-counts.json") -Encoding utf8
if ($matrixCounts.Contains("SPEC_LEGAL_NOT_IMPLEMENTED") -and $matrixCounts["SPEC_LEGAL_NOT_IMPLEMENTED"] -ne 0) {
    throw "authoritative matrix still contains SPEC_LEGAL_NOT_IMPLEMENTED cells"
}
$gapAudit = Get-Content -Raw -LiteralPath (Join-Path $artifact "gap-families.json") | ConvertFrom-Json
if ($gapAudit.matrix_rows -ne 140400 -or $gapAudit.initial_spec_legal_not_implemented -ne 18369 -or $gapAudit.unique_gap_families -ne 5) {
    throw "gap-family audit does not match the recorded initial census"
}
$gapImplemented = ($gapAudit.families | Measure-Object -Property implemented_cells -Sum).Sum
$gapReclassified = ($gapAudit.families | Measure-Object -Property reclassified_cells -Sum).Sum
if ($gapImplemented -ne 85 -or $gapReclassified -ne 18284 -or $gapImplemented + $gapReclassified -ne $gapAudit.initial_spec_legal_not_implemented) {
    throw "gap-family implementation/reclassification totals are inconsistent"
}
$familyAuditMismatches = [System.Collections.Generic.List[string]]::new()
foreach ($family in $gapAudit.families) {
    $initialFamilyRows = @($baselineGapCells | Where-Object reason -eq $family.reason)
    if ($initialFamilyRows.Count -ne $family.initial_cells) {
        $familyAuditMismatches.Add("$($family.id): initial reason count differs")
        continue
    }
    $finalCounts = @{}
    foreach ($initialRow in $initialFamilyRows) {
        $finalRow = $currentRowsByKey[(Get-CellKey $initialRow)]
        if ($null -eq $finalRow) { $familyAuditMismatches.Add("$($family.id): baseline cell disappeared"); continue }
        $finalCounts[$finalRow.verdict] = 1 + ($finalCounts[$finalRow.verdict] | ForEach-Object { $_ })
    }
    foreach ($expected in $family.final_verdicts.PSObject.Properties) {
        if (($finalCounts[$expected.Name] | ForEach-Object { $_ }) -ne $expected.Value) {
            $familyAuditMismatches.Add("$($family.id): $($expected.Name) expected $($expected.Value)")
        }
    }
}
if ($familyAuditMismatches.Count -ne 0) {
    throw "gap-family final verdict mapping differs: $($familyAuditMismatches -join '; ')"
}
$reclassificationRows = Import-Csv -LiteralPath (Join-Path $artifact "reclassifications.csv")
$reclassificationCount = ($reclassificationRows | ForEach-Object { [int]$_.affected_cells } | Measure-Object -Sum).Sum
if ($reclassificationCount -ne 18284) { throw "reclassification audit total is inconsistent" }
$finalDispositionRows = Import-Csv -LiteralPath (Join-Path $artifact "final-gap-disposition.csv")
if ($finalDispositionRows.Count -ne 18369) { throw "final gap disposition artifact does not contain every baseline gap" }
$finalDispositionByKey = @{}
foreach ($row in $finalDispositionRows) {
    $key = Get-CellKey $row
    if ($finalDispositionByKey.ContainsKey($key)) { throw "final gap disposition artifact contains duplicate key $key" }
    $finalDispositionByKey[$key] = $row
}
$dispositionMismatches = [System.Collections.Generic.List[string]]::new()
foreach ($initialRow in $baselineGapCells) {
    $key = Get-CellKey $initialRow
    $disposition = $finalDispositionByKey[$key]
    $currentRow = $currentRowsByKey[$key]
    if ($null -eq $disposition -or $null -eq $currentRow) {
        $dispositionMismatches.Add("missing disposition for $key")
        continue
    }
    $expectedDisposition = if ($currentRow.verdict -eq "SUPPORTED_PASS") { "IMPLEMENTED" } else { "RECLASSIFIED" }
    if ($disposition.original_verdict -ne "SPEC_LEGAL_NOT_IMPLEMENTED" -or $disposition.original_reason -ne $initialRow.reason -or $disposition.final_verdict -ne $currentRow.verdict -or $disposition.disposition -ne $expectedDisposition) {
        $dispositionMismatches.Add("disposition differs for $key")
    }
}
if ($dispositionMismatches.Count -ne 0) { throw "per-cell gap disposition differs: $($dispositionMismatches -join '; ')" }
$implementedDispositionCount = @($finalDispositionRows | Where-Object disposition -eq "IMPLEMENTED").Count
$reclassifiedDispositionCount = @($finalDispositionRows | Where-Object disposition -eq "RECLASSIFIED").Count
if ($implementedDispositionCount -ne 85 -or $reclassifiedDispositionCount -ne 18284) {
    throw "per-cell disposition totals are inconsistent"
}

Invoke-Checked "cargo" @("test", "-p", "jocforge-core", "--quiet", "--locked", "--test", "writer_proof")

$fixture = Join-Path $representatives "fixture.bw64"
$fixtureLfe = Join-Path $representatives "fixture-lfe.bw64"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--locked", "--", "fixture", "--seconds", "1", $fixture)
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--locked", "--", "fixture-directspeakers", "--seconds", "1", $fixtureLfe)

$cases = @(
    [pscustomobject]@{ name = "f1-idx3-skipfield-six"; input = $fixture; profile = "idx3"; carriage = "i0-skipfld"; carriage_debug = "I0SkipField"; emdf = "etsi"; emdf_debug = "EtsiStrict"; bitrate = "1024"; partition = "6"; layout = "5.1"; validation_profile = "auto"; dependent_count = 0; lfe = "absent" },
    [pscustomobject]@{ name = "f2-f3-idx0-skipfield-lfe-2x2x2"; input = $fixtureLfe; profile = "idx0"; carriage = "i0-skipfld"; carriage_debug = "I0SkipField"; emdf = "deployed"; emdf_debug = "DeployedCompat"; bitrate = "1024"; partition = "2+2+2"; layout = "5.1"; validation_profile = "observed-vendor-compat"; dependent_count = 0; lfe = "independent_owned:I0" },
    [pscustomobject]@{ name = "f4-f5-idx1-legacy-deployed-lfe"; input = $fixtureLfe; profile = "idx1"; carriage = "legacy-ac3"; carriage_debug = "LegacyAc3Dependent"; emdf = "deployed"; emdf_debug = "DeployedCompat"; bitrate = "1280"; partition = "6"; layout = "7.1.4"; validation_profile = "observed-vendor-compat"; dependent_count = 1; lfe = "independent_owned:I0" }
)

$results = [System.Collections.Generic.List[object]]::new()
foreach ($case in $cases) {
    $output = Join-Path $representatives "$($case.name).ec3"
    $repeat = Join-Path $representatives "$($case.name).repeat.ec3"
    $encodeArguments = @("run", "-p", "jocforge", "--quiet", "--locked", "--", "encode", $case.input, $output, "--profile", $case.profile, "--carriage", $case.carriage, "--emdf-profile", $case.emdf, "--bitrate", $case.bitrate)
    if ($case.partition -ne "6") { $encodeArguments += @("--block-partition", $case.partition) }
    $encodeLog = Join-Path $artifact "$($case.name).encode.log"
    $encodeExit = Invoke-Captured "cargo" $encodeArguments $encodeLog
    if ($encodeExit -ne 0) { throw "production encode failed for $($case.name)" }
    $encodeText = Get-Content -Raw -LiteralPath $encodeLog
    $expectedTargetBitrate = [int]$case.bitrate * 1000
    if ($encodeText -notlike "*encoded profile=$($case.profile) carriage=$($case.carriage_debug) emdf_profile=$($case.emdf_debug) block_partition=$($case.partition) *target_bitrate=$expectedTargetBitrate*") {
        throw "resolved production plan does not match the requested axes for $($case.name)"
    }
    $expectedLfeState = if ($case.lfe -eq "absent") { "absent" } else { "present" }
    if ($encodeText -notmatch "lfe=$expectedLfeState") {
        throw "resolved production plan does not match the requested LFE state for $($case.name)"
    }
    $repeatArguments = $encodeArguments.Clone()
    $repeatArguments[$repeatArguments.IndexOf($output)] = $repeat
    $repeatLog = Join-Path $artifact "$($case.name).repeat.encode.log"
    $repeatExit = Invoke-Captured "cargo" $repeatArguments $repeatLog
    if ($repeatExit -ne 0) { throw "deterministic repeat encode failed for $($case.name)" }
    $hash = Get-Sha256 $output
    $repeatHash = Get-Sha256 $repeat
    if ($hash -ne $repeatHash) { throw "non-deterministic output for $($case.name)" }

    $inspectLog = Join-Path $artifact "$($case.name).openjoc-inspect.log"
    $inspectExit = Invoke-Captured $pinnedOpenJoc @("inspect", $output) $inspectLog
    $inspectText = Get-Content -Raw -LiteralPath $inspectLog
    $render = Join-Path $representatives "$($case.name).wav"
    $renderLog = Join-Path $artifact "$($case.name).render.log"
    $renderArguments = @("render-joc", $output, "--layout", $case.layout, "--output", $render, "--no-progress", "--overwrite")
    if ($case.validation_profile -ne "auto") { $renderArguments += @("--validation-profile", $case.validation_profile) }
    $renderExit = Invoke-Captured $pinnedOpenJoc $renderArguments $renderLog
    $sourceInspectLog = Join-Path $artifact "$($case.name).openjoc-inspect.json"
    $sourceInspectExit = Invoke-Captured $sourceOpenJoc @("inspect", $output, "--json") $sourceInspectLog
    $sourceInspect = if ($sourceInspectExit -eq 0) { Get-Content -Raw -LiteralPath $sourceInspectLog | ConvertFrom-Json } else { $null }
    $ffmpegLog = Join-Path $artifact "$($case.name).ffmpeg.log"
    $ffmpegExit = Invoke-Captured "ffmpeg" @("-hide_banner", "-v", "error", "-f", "eac3", "-i", $output, "-f", "null", "-") $ffmpegLog
    $malformedMatch = [regex]::Match($inspectText, "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed")
    if (-not $malformedMatch.Success) {
        $malformedMatch = [regex]::Match($inspectText, "skipfld EMDF candidates: \d+ parsed, \d+ non-EMDF, (\d+) malformed")
    }
    $malformed = if ($malformedMatch.Success) { [int]$malformedMatch.Groups[1].Value } else { -1 }
    if ($inspectExit -ne 0 -or $sourceInspectExit -ne 0 -or $renderExit -ne 0 -or $malformed -ne 0 -or $ffmpegExit -ne 0) {
        throw "representative validation failed for $($case.name)"
    }
    if ($sourceInspect.validation.malformed_aus -ne 0 -or -not $sourceInspect.diagnostics.complete -or $null -ne $sourceInspect.diagnostics.first_failure) {
        throw "current OpenJOC Inspector JSON reports a validation failure for $($case.name)"
    }
    $actualEac3 = $sourceInspect.eac3
    if (@($actualEac3.dependent_ids).Count -ne $case.dependent_count) {
        throw "resolved dependent count does not match the requested plan for $($case.name)"
    }
    $expectedInspectorPartition = $case.partition.Replace("+", " ")
    $actualInspectorPartition = @($actualEac3.block_partitions[0].value) -join " "
    if ($actualInspectorPartition -ne $expectedInspectorPartition) {
        throw "resolved block partition does not match the requested plan for $($case.name)"
    }
    if ($actualEac3.legacy_core -ne ($case.carriage -eq "legacy-ac3")) {
        throw "resolved carriage topology does not match the requested plan for $($case.name)"
    }
    if ($actualEac3.lfe_ownership[0].value -ne $case.lfe) {
        throw "resolved LFE ownership does not match the requested plan for $($case.name)"
    }
    if ([math]::Abs([double]$actualEac3.bitrate_bps - $expectedTargetBitrate) -gt 0.5) {
        throw "resolved bitrate does not match the requested plan for $($case.name)"
    }
    $expectedFramesPerAu = $case.partition.Split("+").Count * ($case.dependent_count + 1)
    if (@($actualEac3.frames_per_au | Sort-Object -Unique).Count -ne 1 -or @($actualEac3.frames_per_au)[0] -ne $expectedFramesPerAu) {
        throw "resolved component count is not uniform across all access units for $($case.name)"
    }
    if (@($actualEac3.block_partitions | ForEach-Object { @($_.value) -join " " } | Sort-Object -Unique).Count -ne 1) {
        throw "resolved block partition is not uniform across all access units for $($case.name)"
    }
    if (@($actualEac3.lfe_ownership | ForEach-Object value | Sort-Object -Unique).Count -ne 1) {
        throw "resolved LFE ownership is not uniform across all access units for $($case.name)"
    }
    if ($sourceInspect.joc.payload_occurrences -ne $actualEac3.access_unit_count -or $sourceInspect.emdf.containers -ne $actualEac3.access_unit_count) {
        throw "resolved metadata is not present once per access unit for $($case.name)"
    }
    if ($case.carriage -eq "i0-skipfld" -and $sourceInspect.carriage.skip_observed -le 0) {
        throw "resolved skip-field carriage did not expose a skip-field payload for $($case.name)"
    }
    if ($case.carriage -eq "legacy-ac3" -and $sourceInspect.carriage.aux_parsed -ne $actualEac3.access_unit_count) {
        throw "resolved legacy carriage did not parse one final EMDF per AU for $($case.name)"
    }
    $results.Add([ordered]@{
        name = $case.name
        profile = $case.profile
        carriage = $case.carriage
        emdf_profile = $case.emdf
        partition = $case.partition
        bitrate_kbps = [int]$case.bitrate
        resolved_dependent_count = @($actualEac3.dependent_ids).Count
        resolved_partition = $actualInspectorPartition
        resolved_lfe_ownership = $actualEac3.lfe_ownership[0].value
        resolved_legacy_core = $actualEac3.legacy_core
        resolved_skip_observed = $sourceInspect.carriage.skip_observed
        resolved_joc_owners = @($sourceInspect.joc.owners)
        deterministic = $true
        pinned_openjoc_inspect = $inspectExit -eq 0
        pinned_openjoc_render = $renderExit -eq 0
        pinned_malformed_aus = $malformed
        inspector_json = $sourceInspectExit -eq 0
        inspector_malformed_aus = if ($sourceInspect) { $sourceInspect.validation.malformed_aus } else { -1 }
        inspector_source_revision = $sourceRevision
        source_causality = "pass: source_derived_skip_field_short_writer_uses_the_selected_partition; source_derived_skip_field_short_writer_serializes_independent_lfe_causally; source_derived_skip_field_six_writer_closes_phase_profile_without_ffmpeg_walker; legacy_plan_writer_closes_deployed_and_all_lfe_ownership_modes"
        ffmpeg_decode = $ffmpegExit -eq 0
        output_bytes = (Get-Item -LiteralPath $output).Length
    })
}

$results | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "SPEC_LEGAL_NOT_IMPLEMENTED zero closure evidence written to $artifact"
