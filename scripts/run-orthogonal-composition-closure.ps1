# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\orthogonal-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null
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

function Invoke-Captured([string]$file, [string[]]$arguments, [string]$log) {
    $previous = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & $file @arguments *> $log
    $exit = $LASTEXITCODE
    $ErrorActionPreference = $previous
    return $exit
}

$matrixCsv = Join-Path $root "artifacts\composition-capability-matrix.csv"
$matrixJson = Join-Path $root "artifacts\composition-capability-matrix.json"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "composition-matrix", $matrixCsv, $matrixJson)

$cases = @(
    [pscustomobject]@{ name = "idx0-d0-3x2"; profile = "idx0"; dependents = 1; partition = "3+3"; lfe = "absent"; emdf = "etsi"; layout = "5.1" },
    [pscustomobject]@{ name = "idx2-d1-1x6-deployed"; profile = "idx2"; dependents = 2; partition = "1+1+1+1+1+1"; lfe = "absent"; emdf = "deployed"; layout = "7.1.4" },
    [pscustomobject]@{ name = "idx4-d1-2x3-replacement"; profile = "idx4"; dependents = 2; partition = "2+2+2"; lfe = "replacement"; emdf = "etsi"; layout = "7.1.4" },
    [pscustomobject]@{ name = "legal-monster-idx4-d0-d4-3x2-supplement"; profile = "idx4"; dependents = 5; partition = "3+3"; lfe = "supplement"; emdf = "etsi"; layout = "7.1.4" },
    [pscustomobject]@{ name = "legal-monster-idx4-d0-d5-3x2-absent"; profile = "idx4"; dependents = 6; partition = "3+3"; lfe = "absent"; emdf = "etsi"; layout = "7.1.4" }
)

$rows = [System.Collections.Generic.List[object]]::new()
foreach ($case in $cases) {
    $output = Join-Path $artifact "$($case.name).ec3"
    $arguments = @("run", "-p", "jocforge", "--quiet", "--", "conformance-short-multi-dependent", $case.profile, "$($case.dependents)", $case.partition, $output, "--emdf", $case.emdf, "--lfe", $case.lfe)
    Invoke-Checked "cargo" $arguments

    $inspectLog = Join-Path $artifact "$($case.name).openjoc-inspect.log"
    $inspectExit = Invoke-Captured $pinnedOpenJoc @("inspect", $output) $inspectLog
    $pinnedInspectText = if (Test-Path -LiteralPath $inspectLog) { Get-Content -Raw -LiteralPath $inspectLog } else { "" }
    $pinnedMalformedMatch = [regex]::Match($pinnedInspectText, "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed")
    $pinnedMalformed = if ($pinnedMalformedMatch.Success) { [int]$pinnedMalformedMatch.Groups[1].Value } else { -1 }
    $render = Join-Path $artifact "$($case.name).wav"
    $renderLog = Join-Path $artifact "$($case.name).render.log"
    $renderExit = Invoke-Captured $pinnedOpenJoc @("render-joc", $output, "--layout", $case.layout, "--output", $render, "--no-progress", "--overwrite") $renderLog
    $sourceInspectPath = Join-Path $artifact "$($case.name).openjoc-inspect.json"
    $sourceInspectExit = Invoke-Captured $sourceOpenJoc @("inspect", $output, "--json") $sourceInspectPath
    $sourceInspect = if ($sourceInspectExit -eq 0) {
        Get-Content -Raw -LiteralPath $sourceInspectPath | ConvertFrom-Json
    } else { $null }
    $ffmpegLog = Join-Path $artifact "$($case.name).ffmpeg.log"
    $ffmpegExit = Invoke-Captured "ffmpeg" @("-hide_banner", "-v", "error", "-f", "eac3", "-i", $output, "-f", "null", "-") $ffmpegLog
    $strictStatus = if ($pinnedInspectText -match "profile: ETSI_STRICT\s+result: normative_compliant") { "pass" } else { "fail" }
    $deployedStatus = if ($pinnedInspectText -match "profile: OBSERVED_VENDOR_COMPAT\s+result: (normative_compliant|accepted_with_deviation)") { "pass" } else { "fail" }
    if ($inspectExit -ne 0 -or $renderExit -ne 0) {
        throw "OpenJOC validation command failed for $($case.name)"
    }
    if ($pinnedMalformed -ne 0) {
        throw "frozen OpenJOC inspect found malformed EMDF AUs for $($case.name)"
    }
    if ($case.emdf -eq "etsi" -and $strictStatus -ne "pass") {
        throw "ETSI strict validation failed for $($case.name)"
    }
    if ($case.emdf -eq "deployed" -and $deployedStatus -ne "pass") {
        throw "deployed compatibility validation failed for $($case.name)"
    }
    $normativeEvidencePass = $inspectExit -eq 0 -and $renderExit -eq 0 -and $pinnedMalformed -eq 0 -and (($case.emdf -eq "etsi" -and $strictStatus -eq "pass") -or ($case.emdf -eq "deployed" -and $deployedStatus -eq "pass"))
    $ffmpegStatus = if ($ffmpegExit -eq 0) {
        "PASS"
    } elseif ($normativeEvidencePass -and $case.dependents -gt 1) {
        "UNSUPPORTED_DECODER_FEATURE"
    } else {
        "FAIL_STREAM_ERROR"
    }
    $rows.Add([ordered]@{
        name = $case.name
        profile = $case.profile
        dependent_count = $case.dependents
        partition = $case.partition
        lfe_ownership = $case.lfe
        emdf_profile = $case.emdf
        pinned_openjoc_inspect = $inspectExit -eq 0
        pinned_openjoc_render = $renderExit -eq 0
        authoritative_openjoc = "frozen validator manifest"
        inspector_json = $sourceInspectExit -eq 0
        inspector_source_revision = $sourceRevision
        pinned_malformed_aus = $pinnedMalformed
        inspector_malformed_aus = if ($sourceInspect) { $sourceInspect.validation.malformed_aus } else { -1 }
        etsi_strict = $strictStatus
        deployed_compatibility = $deployedStatus
        ffmpeg_interoperability = $ffmpegStatus
        output_bytes = (Get-Item -LiteralPath $output).Length
    })
}

$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "orthogonal composition closure evidence written to $artifact"
