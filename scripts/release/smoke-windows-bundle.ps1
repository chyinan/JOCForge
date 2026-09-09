# pattern: Imperative Shell

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$ZipPath,
    [string]$ReportPath,
    [string]$ReleaseArtifactPath,
    [string]$OpenJocPath,
    [switch]$RequireOpenJoc
)

$ErrorActionPreference = "Stop"
$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

function Get-Sha256([string]$Path) {
    $algorithm = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($Path)
    try {
        return (($algorithm.ComputeHash($stream) | ForEach-Object { $_.ToString("x2") }) -join "")
    }
    finally {
        $stream.Dispose()
        $algorithm.Dispose()
    }
}

function Invoke-Checked([string]$File, [string[]]$Arguments) {
    & $File @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed with exit code $LASTEXITCODE`: $File $($Arguments -join ' ')"
    }
}

function Invoke-OpenJocRawQa([string]$EncodedPath, [string]$Layout, [string]$Name) {
    if ([string]::IsNullOrWhiteSpace($OpenJocPath)) {
        return
    }
    Invoke-Checked $OpenJocPath @("inspect", $EncodedPath)
    $render = Join-Path $work "$Name-openjoc.wav"
    Invoke-Checked $OpenJocPath @("render-joc", $EncodedPath, "--layout", $Layout, "--output", $render, "--no-progress", "--overwrite")
}

if (!(Test-Path -LiteralPath $ZipPath -PathType Leaf)) {
    throw "release ZIP does not exist: $ZipPath"
}
if ($RequireOpenJoc -and [string]::IsNullOrWhiteSpace($OpenJocPath)) {
    throw "pinned OpenJOC is required for release semantic QA"
}
if (![string]::IsNullOrWhiteSpace($OpenJocPath) -and !(Test-Path -LiteralPath $OpenJocPath -PathType Leaf)) {
    throw "OpenJOC executable does not exist: $OpenJocPath"
}
if (![string]::IsNullOrWhiteSpace($OpenJocPath)) {
    . (Join-Path $repositoryRoot "scripts\openjoc-validator-pin.ps1")
    Assert-OpenJocValidatorPin $repositoryRoot $OpenJocPath | Out-Null
}
$pythonPath = $null
if (![string]::IsNullOrWhiteSpace($OpenJocPath)) {
    $pythonCommand = Get-Command python.exe -ErrorAction SilentlyContinue
    if ($null -eq $pythonCommand) {
        throw "Python is required for the Flat-7.X 8/8 release semantic gate"
    }
    $pythonPath = $pythonCommand.Source
}

$tempRoot = Join-Path ([IO.Path]::GetTempPath()) "jocforge-release-smoke-$PID"
if (Test-Path -LiteralPath $tempRoot) {
    throw "smoke directory already exists: $tempRoot"
}
New-Item -ItemType Directory -Path $tempRoot | Out-Null
$oldPath = $env:Path
$oldLocation = Get-Location
$report = [ordered]@{
    zip_sha256 = Get-Sha256 $ZipPath
    bundled_ffmpeg_actually_used = $false
    system_ffmpeg_required = $true
    openjoc_qa = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "NOT_RUN" } else { "PENDING" }
    profiles = @()
    lfe = "NOT_RUN"
    lfe_ffmpeg = "NOT_RUN"
    short_block = "NOT_RUN"
    short_block_ffmpeg = "NOT_RUN"
    multi_dependent = "NOT_RUN"
    multi_dependent_ffmpeg = "NOT_RUN"
    legacy_core = "NOT_RUN"
    legacy_core_ffmpeg = "NOT_RUN"
    cmaf = "NOT_RUN"
    cmaf_ffprobe = "NOT_RUN"
    flat7x_audio_semantic_identity = "NOT_RUN"
}

try {
    Expand-Archive -LiteralPath $ZipPath -DestinationPath $tempRoot
    $bundleRoot = $tempRoot
    $jocforge = Join-Path $bundleRoot "jocforge.exe"
    $ffmpeg = Join-Path $bundleRoot "ffmpeg.exe"
    $ffprobe = Join-Path $bundleRoot "ffprobe.exe"
    $manifestPath = Join-Path $bundleRoot "manifest.json"
    $checksumsPath = Join-Path $bundleRoot "SHA256SUMS.txt"
    foreach ($required in @(@($jocforge, "jocforge.exe"), @($ffmpeg, "ffmpeg.exe"), @($ffprobe, "ffprobe.exe"))) {
        if (!(Test-Path -LiteralPath $required[0] -PathType Leaf)) {
            throw "bundle is missing $($required[1])"
        }
    }
    foreach ($required in @(@($manifestPath, "manifest.json"), @($checksumsPath, "SHA256SUMS.txt"))) {
        if (!(Test-Path -LiteralPath $required[0] -PathType Leaf)) {
            throw "bundle is missing $($required[1])"
        }
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $pin = Get-Content -LiteralPath (Join-Path $repositoryRoot "release\ffmpeg-pin.json") -Raw | ConvertFrom-Json
    if ($manifest.ffmpeg.version -ne $pin.ffmpeg.version -or
        $manifest.ffmpeg.tag -ne $pin.ffmpeg.tag -or
        $manifest.ffmpeg.commit -ne $pin.ffmpeg.commit -or
        $manifest.ffmpeg.source_url -ne $pin.ffmpeg.source_url -or
        $manifest.ffmpeg.source_sha256 -ne $pin.ffmpeg.source_sha256) {
        throw "bundle FFmpeg metadata does not match the checked-in release pin"
    }
    $actualFfmpegHash = Get-Sha256 $ffmpeg
    $actualFfprobeHash = Get-Sha256 $ffprobe
    $actualJocforgeHash = Get-Sha256 $jocforge
    $sourceArchive = Get-ChildItem -LiteralPath (Join-Path $bundleRoot "licenses\FFmpeg") -Filter "ffmpeg-*.tar.xz" -File | Select-Object -First 1
    if ($null -eq $sourceArchive) {
        throw "bundle is missing the FFmpeg source archive"
    }
    $actualSourceHash = Get-Sha256 $sourceArchive.FullName
    if ($manifest.jocforge.sha256 -ne $actualJocforgeHash -or
        $manifest.ffmpeg.executable_sha256 -ne $actualFfmpegHash -or
        $manifest.ffprobe.executable_sha256 -ne $actualFfprobeHash -or
        $manifest.ffmpeg.source_sha256 -ne $actualSourceHash) {
        throw "bundle manifest does not match extracted executable/source hashes"
    }
    $checksumText = Get-Content -LiteralPath $checksumsPath -Raw
    $bundleName = [string]$manifest.bundle
    $sourceRelative = "$bundleName/licenses/FFmpeg/$($sourceArchive.Name)"
    foreach ($expected in @(
        "$actualJocforgeHash  jocforge.exe",
        "$actualFfmpegHash  ffmpeg.exe",
        "$actualFfprobeHash  ffprobe.exe",
        "$actualSourceHash  licenses/FFmpeg/$($sourceArchive.Name)"
    )) {
        if ($checksumText -notmatch [regex]::Escape($expected)) {
            throw "bundle SHA256SUMS.txt does not match extracted file hashes: $expected"
        }
    }
    $artifact = $null
    if (![string]::IsNullOrWhiteSpace($ReleaseArtifactPath)) {
        if (!(Test-Path -LiteralPath $ReleaseArtifactPath -PathType Leaf)) {
            throw "release-artifact.json does not exist: $ReleaseArtifactPath"
        }
        $artifact = Get-Content -LiteralPath $ReleaseArtifactPath -Raw | ConvertFrom-Json
        $zipHash = Get-Sha256 $ZipPath
        if ($artifact.zip_sha256 -ne $zipHash -or
            $artifact.jocforge_sha256 -ne $actualJocforgeHash -or
            $artifact.ffmpeg_sha256 -ne $actualFfmpegHash -or
            $artifact.ffprobe_sha256 -ne $actualFfprobeHash -or
            $artifact.ffmpeg_source_sha256 -ne $actualSourceHash) {
            throw "release-artifact.json does not match the final ZIP and extracted files"
        }
        $externalChecksumsPath = Join-Path (Split-Path -Parent $ReleaseArtifactPath) "SHA256SUMS.txt"
        if (!(Test-Path -LiteralPath $externalChecksumsPath -PathType Leaf)) {
            throw "external SHA256SUMS.txt does not exist: $externalChecksumsPath"
        }
        $externalChecksums = Get-Content -LiteralPath $externalChecksumsPath -Raw
        foreach ($expected in @(
            "$zipHash  $([IO.Path]::GetFileName($ZipPath))",
            "$actualJocforgeHash  $bundleName/jocforge.exe",
            "$actualFfmpegHash  $bundleName/ffmpeg.exe",
            "$actualFfprobeHash  $bundleName/ffprobe.exe",
            "$actualSourceHash  $sourceRelative"
        )) {
            if ($externalChecksums -notmatch [regex]::Escape($expected)) {
                throw "external SHA256SUMS.txt does not contain expected hash: $expected"
            }
        }
        $report.zip_sha256 = $zipHash
    }
    $report.jocforge_sha256 = $actualJocforgeHash

    $systemRoot = [Environment]::GetEnvironmentVariable("SystemRoot")
    $env:Path = "$bundleRoot;$systemRoot\System32;$systemRoot"
    $resolvedFfmpeg = (& where.exe ffmpeg.exe 2>$null | Select-Object -First 1).Trim()
    if ([string]::IsNullOrWhiteSpace($resolvedFfmpeg)) {
        throw "bundled FFmpeg was not discoverable from the isolated PATH"
    }
    if ([IO.Path]::GetFullPath($resolvedFfmpeg) -ne [IO.Path]::GetFullPath($ffmpeg)) {
        throw "isolated PATH resolved FFmpeg outside the extracted bundle: $resolvedFfmpeg"
    }
    $report.bundled_ffmpeg_actually_used = $true
    $report.system_ffmpeg_required = $false
    $report.ffmpeg_version = ((& $ffmpeg -version 2>&1) -join "`n").Trim()
    $report.ffprobe_version = ((& $ffprobe -version 2>&1) -join "`n").Trim()
    $report.ffmpeg_path = [IO.Path]::GetFullPath($ffmpeg)
    $report.ffprobe_path = [IO.Path]::GetFullPath($ffprobe)
    $report.ffmpeg_sha256 = $actualFfmpegHash
    $report.ffprobe_sha256 = $actualFfprobeHash

    Set-Location $bundleRoot
    $work = Join-Path $bundleRoot "smoke-work-测试"
    New-Item -ItemType Directory -Path $work | Out-Null
    $fixture = Join-Path $work "synthetic.bw64"
    Invoke-Checked $jocforge @("fixture", "--seconds", "1", $fixture)

    $profiles = @(
        @{ name = "idx0"; layout = "2.0" },
        @{ name = "idx1"; layout = "7.1.4" },
        @{ name = "idx2"; layout = "7.1.4" },
        @{ name = "idx3"; layout = "2.0" },
        @{ name = "idx4"; layout = "7.1.4" }
    )
    foreach ($profile in $profiles) {
        $output = Join-Path $work "$($profile.name).ec3"
        Invoke-Checked $jocforge @("encode", $fixture, $output, "--profile", $profile.name)
        Invoke-Checked $ffmpeg @("-hide_banner", "-v", "error", "-f", "eac3", "-i", $output, "-f", "null", "-")
        Invoke-Checked $ffprobe @("-v", "error", "-show_entries", "stream=codec_name", "-of", "json", $output)
        $report.profile_ffmpeg = "PASS"
        $report.profile_openjoc = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "NOT_RUN" } else { "PASS" }
        Invoke-OpenJocRawQa $output $profile.layout $profile.name
        $report.profiles += $profile.name
    }

    $lfeFixture = Join-Path $work "lfe.bw64"
    $lfeOutput = Join-Path $work "lfe.ec3"
    Invoke-Checked $jocforge @("fixture-directspeakers", "--seconds", "1", $lfeFixture)
    Invoke-Checked $jocforge @("encode", $lfeFixture, $lfeOutput, "--profile", "idx1")
    Invoke-Checked $ffmpeg @("-hide_banner", "-v", "error", "-f", "eac3", "-i", $lfeOutput, "-f", "null", "-")
    $report.lfe_ffmpeg = "PASS"
    Invoke-OpenJocRawQa $lfeOutput "7.1" "lfe"
    $report.lfe = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "NOT_RUN" } else { "PASS" }

    $shortOutput = Join-Path $work "short-3-3.ec3"
    Invoke-Checked $jocforge @("encode", $fixture, $shortOutput, "--profile", "idx1", "--block-partition", "3+3", "--bitrate", "1664")
    Invoke-Checked $ffmpeg @("-hide_banner", "-v", "error", "-f", "eac3", "-i", $shortOutput, "-f", "null", "-")
    $report.short_block_ffmpeg = "PASS"
    Invoke-OpenJocRawQa $shortOutput "7.1" "short-block"
    $report.short_block = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "NOT_RUN" } else { "PASS" }

    $multiOutput = Join-Path $work "multi-dependent.ec3"
    Invoke-Checked $jocforge @("conformance-multi-dependent", "idx0", "2", $multiOutput)
    Invoke-Checked $ffmpeg @("-hide_banner", "-v", "error", "-f", "eac3", "-i", $multiOutput, "-f", "null", "-")
    $report.multi_dependent_ffmpeg = "PASS"
    Invoke-OpenJocRawQa $multiOutput "2.0" "multi-dependent"
    $report.multi_dependent = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "NOT_RUN" } else { "PASS" }

    $legacyOutput = Join-Path $work "legacy.ec3"
    Invoke-Checked $jocforge @("encode", $fixture, $legacyOutput, "--profile", "idx1", "--carriage", "legacy-ac3", "--bitrate", "1280")
    Invoke-Checked $ffmpeg @("-hide_banner", "-v", "error", "-f", "eac3", "-i", $legacyOutput, "-f", "null", "-")
    $report.legacy_core_ffmpeg = "PASS"
    Invoke-OpenJocRawQa $legacyOutput "7.1" "legacy-core"
    $report.legacy_core = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "NOT_RUN" } else { "PASS" }

    $rawOutput = Join-Path $work "cmaf-source.ec3"
    $cmafOutput = Join-Path $work "cmaf.mp4"
    Invoke-Checked $jocforge @("proof", "idx1", $rawOutput)
    Invoke-Checked $jocforge @("conformance-cmaf", "idx1", $rawOutput, $cmafOutput)
    Invoke-Checked $ffprobe @("-v", "error", "-show_entries", "stream=codec_name,codec_tag_string", "-of", "json", $cmafOutput)
    $report.cmaf_ffprobe = "PASS"
    if (![string]::IsNullOrWhiteSpace($OpenJocPath)) {
        $cmafDecode = Join-Path $work "cmaf-openjoc-decode"
        Invoke-Checked $OpenJocPath @("decode", $cmafOutput, "--internal-base", "-o", $cmafDecode)
    }
    $report.cmaf = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "NOT_RUN" } else { "PASS" }

    if (![string]::IsNullOrWhiteSpace($OpenJocPath)) {
        $flatBase = Join-Path $work "flat7x-base.bw64"
        $flatRoot = Join-Path $work "flat7x-identity"
        New-Item -ItemType Directory -Path $flatRoot | Out-Null
        Invoke-Checked $jocforge @("fixture", "--seconds", "1", "--objects", "7", $flatBase)
        $fixtureScript = Join-Path $repositoryRoot "scripts\flat7x-audit-fixtures.py"
        $analysisScript = Join-Path $repositoryRoot "scripts\analyze-flat7x-semantic-audit.py"
        foreach ($mode in @("full", "L", "R", "C", "Ls", "Rs", "Lrs", "Rrs")) {
            $identityFixture = Join-Path $flatRoot "$mode.bw64"
            $identityOutput = Join-Path $flatRoot "identity-$mode.ec3"
            $identityRender = Join-Path $flatRoot "identity-$mode.wav"
            Invoke-Checked $pythonPath @($fixtureScript, $flatBase, $identityFixture, $mode)
            Invoke-Checked $jocforge @("encode", $identityFixture, $identityOutput, "--profile", "idx1")
            Invoke-Checked $OpenJocPath @("inspect", $identityOutput)
            Invoke-Checked $OpenJocPath @("render-joc", $identityOutput, "--layout", "7.1", "--output", $identityRender, "--no-progress", "--overwrite")
        }
        $lfeIdentityFixture = Join-Path $flatRoot "LFE.bw64"
        $lfeIdentityOutput = Join-Path $flatRoot "identity-LFE.ec3"
        $lfeIdentityRender = Join-Path $flatRoot "identity-LFE.wav"
        Invoke-Checked $pythonPath @($fixtureScript, $lfeFixture, $lfeIdentityFixture, "lfe")
        Invoke-Checked $jocforge @("encode", $lfeIdentityFixture, $lfeIdentityOutput, "--profile", "idx1")
        Invoke-Checked $OpenJocPath @("inspect", $lfeIdentityOutput)
        Invoke-Checked $OpenJocPath @("render-joc", $lfeIdentityOutput, "--layout", "7.1", "--output", $lfeIdentityRender, "--no-progress", "--overwrite")
        Invoke-Checked $pythonPath @($analysisScript, $flatRoot)
        $report.flat7x_audio_semantic_identity = "PASS"
    }
    if (![string]::IsNullOrWhiteSpace($OpenJocPath)) {
        $report.openjoc_qa = "PASS"
    }
    elseif ($RequireOpenJoc) {
        throw "pinned OpenJOC semantic QA was required but not run"
    }
    $report.release_smoke = "PASS"
    $report.release_semantic_qa = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "NOT_RUN" } else { "PASS" }
    $report.binary_distribution_license_gate = if ([string]::IsNullOrWhiteSpace($OpenJocPath)) { "REVIEW_REQUIRED_UNTIL_OPENJOC" } else { "PASS" }
    if ($null -ne $artifact) {
        $artifact | Add-Member -NotePropertyName release_smoke -NotePropertyValue $report.release_smoke -Force
        $artifact | Add-Member -NotePropertyName release_semantic_qa -NotePropertyValue $report.release_semantic_qa -Force
        $artifact | Add-Member -NotePropertyName bundled_ffmpeg_actually_used -NotePropertyValue $report.bundled_ffmpeg_actually_used -Force
        $artifact | Add-Member -NotePropertyName system_ffmpeg_required -NotePropertyValue $report.system_ffmpeg_required -Force
        $artifact | Add-Member -NotePropertyName binary_distribution_license_gate -NotePropertyValue $report.binary_distribution_license_gate -Force
        $artifact | Add-Member -NotePropertyName smoke_report -NotePropertyValue ([IO.Path]::GetFileName($ReportPath)) -Force
        $artifact | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $ReleaseArtifactPath -Encoding utf8
    }
}
finally {
    Set-Location $oldLocation
    $env:Path = $oldPath
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}

if (![string]::IsNullOrWhiteSpace($ReportPath)) {
    $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $ReportPath -Encoding utf8
}
Write-Output ($report | ConvertTo-Json -Depth 8)
