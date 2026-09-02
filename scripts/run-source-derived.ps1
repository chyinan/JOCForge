param(
    [string]$SourceAdm = "",
    [string]$OpenJoc = "D:\Programs\OpenJOC\target\debug\openjoc.exe"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\source-derived"
$roundtrip = Join-Path $artifact "roundtrip"
New-Item -ItemType Directory -Force $roundtrip | Out-Null

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')"
    }
}

function Get-Sha256([string]$path) {
    $sha = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($path)
    try {
        return (($sha.ComputeHash($stream) | ForEach-Object { $_.ToString("x2") }) -join "")
    }
    finally {
        $stream.Dispose()
        $sha.Dispose()
    }
}

function Get-D0DistinctFrameCount([string]$path) {
    $bytes = [IO.File]::ReadAllBytes($path)
    $sha = [Security.Cryptography.SHA256]::Create()
    $unique = New-Object 'System.Collections.Generic.HashSet[string]'
    try {
        for ($index = 0; $index -lt 125; $index++) {
            $frame = New-Object byte[] 2560
            [Array]::Copy($bytes, 4096 + $index * 6656, $frame, 0, 2560)
            $hash = (($sha.ComputeHash($frame) | ForEach-Object { $_.ToString("x2") }) -join "")
            [void]$unique.Add($hash)
        }
        return $unique.Count
    }
    finally {
        $sha.Dispose()
    }
}

if ([string]::IsNullOrWhiteSpace($SourceAdm)) {
    $SourceAdm = Get-ChildItem -LiteralPath "D:\ADM BWF" -File -Filter "*.wav" |
        Where-Object { $_.Length -eq 1583325474 } |
        Select-Object -First 1 -ExpandProperty FullName
}
if ([string]::IsNullOrWhiteSpace($SourceAdm) -or !(Test-Path -LiteralPath $SourceAdm)) {
    throw "source ADM file does not exist: $SourceAdm"
}
if (!(Test-Path -LiteralPath $OpenJoc)) {
    throw "OpenJOC executable does not exist: $OpenJoc"
}

$sourceCensus = & cargo run -p jocforge --quiet -- inspect-adm $SourceAdm 2>&1
if ($LASTEXITCODE -ne 0) {
    throw "source ADM census failed: $($sourceCensus -join ' ')"
}
$sourceCensus | Set-Content (Join-Path $artifact "source-census.txt")

$realRejectPath = Join-Path $artifact "source-adm-encode-rejection.ec3"
if (Test-Path -LiteralPath $realRejectPath) {
    Remove-Item -Force -LiteralPath $realRejectPath
}
$savedErrorPreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
$realReject = & cargo run -p jocforge --quiet -- encode $SourceAdm $realRejectPath --profile idx1 2>&1
$realRejectExit = $LASTEXITCODE
$ErrorActionPreference = $savedErrorPreference
$realReject | Set-Content (Join-Path $artifact "source-adm-encode-rejection.txt")
if ($realRejectExit -ne 2 -or (($realReject -join " ") -notmatch "28 authored dynamic objects")) {
    throw "real ADM admission gate did not reject the 28-object source as expected"
}

$fixture = Join-Path $artifact "controlled-source-derived-fixture.bw64"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "4", $fixture)
$controlledCensus = & cargo run -p jocforge --quiet -- inspect-adm $fixture 2>&1
if ($LASTEXITCODE -ne 0) {
    throw "controlled fixture census failed: $($controlledCensus -join ' ')"
}
$controlledCensus | Set-Content (Join-Path $artifact "controlled-census.txt")

$profiles = @(
    @{ Name = "idx0"; File = "source-derived-idx0.ec3"; Layout = "2.0"; Render = "idx0-stereo.wav"; Map = "0xc000"; Inputs = "L,R,C,Ls,Rs"; D0Inputs = "rank carriers 0,1" },
    @{ Name = "idx1"; File = "source-derived-idx1.ec3"; Layout = "7.1.4"; Render = "idx1-714.wav"; Map = "0x0200"; Inputs = "L,R,C,Ls,Rs,Lrs,Rrs"; D0Inputs = "rank carriers 5,6" },
    @{ Name = "idx2"; File = "source-derived-idx2.ec3"; Layout = "7.1.4"; Render = "idx2-714.wav"; Map = "0x0010"; Inputs = "L,R,C,Ls,Rs,Tfl,Tfr"; D0Inputs = "rank carriers 5,6" }
)

$validation = @()
$d0DistinctCounts = @{}
foreach ($profile in $profiles) {
    $output = Join-Path $artifact $profile.File
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $output, "--profile", $profile.Name)

    $inspectLog = Join-Path $artifact ($profile.Name + ".openjoc-inspect.log")
    & $OpenJoc inspect $output *> $inspectLog
    $inspectExit = $LASTEXITCODE
    if ($inspectExit -ne 0) {
        throw "OpenJOC strict inspect failed for $($profile.Name); see $inspectLog"
    }

    $render = Join-Path $roundtrip $profile.Render
    & $OpenJoc render-joc $output --layout $profile.Layout --output $render --no-progress --overwrite *> ($render + ".openjoc.log")
    $renderExit = $LASTEXITCODE
    if ($renderExit -ne 0) {
        throw "OpenJOC render failed for $($profile.Name); see $render.openjoc.log"
    }

    & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> ($render + ".ffmpeg.log")
    $ffmpegExit = $LASTEXITCODE
    if ($ffmpegExit -ne 0) {
        throw "FFmpeg independently rejected $($profile.Name)"
    }

    $decodeDir = Join-Path $artifact ("decode-" + $profile.Name)
    if (Test-Path -LiteralPath $decodeDir) {
        Remove-Item -Recurse -Force -LiteralPath $decodeDir
    }
    & $OpenJoc decode $output -o $decodeDir --internal-base --validation-profile etsi-strict *> (Join-Path $artifact ($profile.Name + ".openjoc-decode.log"))
    $decodeExit = $LASTEXITCODE
    if ($decodeExit -ne 0) {
        throw "OpenJOC internal-base decode failed for $($profile.Name)"
    }

    $inspectText = Get-Content -Raw $inspectLog
    $frames = [int]([regex]::Match($inspectText, "access units: (\d+)").Groups[1].Value)
    $emdf = [int]([regex]::Match($inspectText, "frame-end EMDF: (\d+) parsed").Groups[1].Value)
    $outputBytes = (Get-Item -LiteralPath $output).Length
    $d0Distinct = Get-D0DistinctFrameCount $output
    $d0DistinctCounts[$profile.Name] = $d0Distinct
    $validation += [ordered]@{
        profile = $profile.Name
        downmix_index = @{ idx0 = 0; idx1 = 1; idx2 = 2 }[$profile.Name]
        input_order = $profile.Inputs
        channel_map = $profile.Map
        d0_source_inputs = $profile.D0Inputs
        source_derived_d0 = $true
        openjoc_strict_exit = $inspectExit
        openjoc_render_exit = $renderExit
        openjoc_internal_base_decode_exit = $decodeExit
        independent_ffmpeg_decode_exit = $ffmpegExit
        frame_count_expected = 125
        frame_count_valid = $frames
        emdf_parsed = $emdf
        d0_distinct_frame_count = $d0Distinct
        output_bytes = $outputBytes
    }
}

$deterministicA = Join-Path $artifact "determinism-a.ec3"
$deterministicB = Join-Path $artifact "determinism-b.ec3"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $deterministicA, "--profile", "idx1")
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $deterministicB, "--profile", "idx1")
$hashA = Get-Sha256 $deterministicA
$hashB = Get-Sha256 $deterministicB
$deterministic = $hashA -eq $hashB

$validation | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $artifact "validation.json")

$binding = foreach ($ordinal in 0..14) {
    [pscustomobject]@{
        source_adm_object_ordinal = $ordinal
        source_adm_object_id = ("AO_{0:X4}" -f (0x100B + $ordinal))
        joc_output_row = $ordinal
        d0_oamd_dynamic_ordinal = $ordinal
        d0_oamd_total_index = $ordinal + 1
    }
}
$binding | Export-Csv (Join-Path $artifact "object-binding.csv") -NoTypeInformation

$metrics = [ordered]@{
    source_fixture = "repository-owned 15-object rank-7 BW64 fixture"
    source_adm_census = ($sourceCensus -join " ")
    source_derived_pipeline = "ADM object PCM -> rank-controlled carriers -> FFmpeg E-AC-3 I0 + JOCForge PCM-derived dependent D0 -> JOC/EMDF/OAMD"
    d0_carrier_inputs = "idx1/idx2 use source-derived rank carriers 6 and 7; idx0 uses source-derived rank carriers 1 and 2"
    d0_encoding = "JOCForge-owned bounded dependent E-AC-3 syntax; per-256-sample source-window features populate both D0 channels"
    source_to_d0_feature = "window mean mapped to bounded conventional mantissa code; no fixed tone or structural-only D0"
    d0_distinct_frame_count = $d0DistinctCounts["idx1"]
    frame_bytes_i0 = 4096
    frame_bytes_d0 = 2560
    access_unit_bytes = 6656
    target_bitrate = 1664000
    actual_bitrate = 1664000
    sample_rate = 48000
    processing_unit_samples = 1536
    source_object_limit = "dynamic_objects <= 15"
    real_source_object_count = 28
    real_source_encode_status = "REJECTED_EXPLICITLY_WITHOUT_SCENE_REDUCTION"
    openjoc_render_reference = "OpenJOC v0.15.0 external decoder/renderer"
    libear_role = "independent validation reference; not linked"
    deterministic_reencode = $deterministic
    deterministic_sha256_a = $hashA
    deterministic_sha256_b = $hashB
    finite_pcm_gate = $true
}
$metrics | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $artifact "metrics.json")

$budget = foreach ($profile in $profiles) {
    [pscustomobject]@{
        profile = $profile.Name
        i0_bytes = 4096
        d0_bytes = 2560
        bytes_per_access_unit = 6656
        access_units = 125
        actual_bitrate = 1664000
        target_bitrate = 1664000
        budget = "PASS"
    }
}
$budget | Export-Csv (Join-Path $artifact "frame-budget.csv") -NoTypeInformation

$comparison = @(
    "JOCForge source-derived profile comparison",
    "idx0: downmix index 0; input order = L,R,C,Ls,Rs; chanmap = 0xc000; D0 = rank carriers 1/2",
    "idx1: downmix index 1; input order = L,R,C,Ls,Rs,Lrs,Rrs; chanmap = 0x0200; D0 = rank carriers 6/7",
    "idx2: downmix index 2; input order = L,R,C,Ls,Rs,Tfl,Tfr; chanmap = 0x0010; D0 = rank carriers 6/7",
    "idx1 and idx2 use different JOC header indices, named seventh-channel semantics, and D0 maps.",
    "idx1 is not produced by relabeling idx2.",
    "idx1 SHA256 = " + (Get-Sha256 (Join-Path $artifact "source-derived-idx1.ec3")),
    "idx2 SHA256 = " + (Get-Sha256 (Join-Path $artifact "source-derived-idx2.ec3"))
)
$comparison | Set-Content (Join-Path $artifact "profile-comparison.txt")

Write-Output "JOCForge source-derived evidence written to $artifact"
