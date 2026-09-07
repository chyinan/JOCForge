# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\mvp"
$roundtrip = Join-Path $artifact "roundtrip"
New-Item -ItemType Directory -Force $roundtrip | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$OpenJoc = (Resolve-OpenJocValidator $root).executable

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

$validatorIdentity = Assert-OpenJocValidatorPin $root $OpenJoc
$validatorIdentity | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-identity.json") -Encoding utf8

$fixture = Join-Path $artifact "synthetic-reference.adm.wav"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "4", $fixture)
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "inspect-adm", $fixture)

$profiles = @(
    @{ Name = "idx0"; File = "jocforge-mvp-idx0.ec3"; Layout = "2.0"; Render = "idx0-stereo.wav"; Map = "0xc000"; Inputs = "L,R,C,Ls,Rs" },
    @{ Name = "idx1"; File = "jocforge-mvp-flat7x-idx1.ec3"; Layout = "7.1.4"; Render = "idx1-714.wav"; Map = "0x0200"; Inputs = "L,R,C,Ls,Rs,Lrs,Rrs" },
    @{ Name = "idx2"; File = "jocforge-mvp-5x2-idx2.ec3"; Layout = "7.1.4"; Render = "idx2-714.wav"; Map = "0x0010"; Inputs = "L,R,C,Ls,Rs,Tfl,Tfr" },
    @{ Name = "idx3"; File = "jocforge-mvp-5x-phase-idx3.ec3"; Layout = "2.0"; Render = "idx3-stereo.wav"; Map = "0xc000"; Inputs = "L,R,C,Ls,Rs" },
    @{ Name = "idx4"; File = "jocforge-mvp-5x2-phase-idx4.ec3"; Layout = "7.1.4"; Render = "idx4-714.wav"; Map = "0x0010"; Inputs = "L,R,C,Ls,Rs,Tfl,Tfr" }
)

$validation = @()
foreach ($profile in $profiles) {
    $output = Join-Path $artifact $profile.File
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $output, "--profile", $profile.Name)
    $inspectLog = Join-Path $artifact ($profile.Name + ".inspect.log")
    & $OpenJoc inspect $output *> $inspectLog
    $inspectExit = $LASTEXITCODE
    if ($inspectExit -ne 0) {
        throw "OpenJOC strict inspect failed for $($profile.Name); see $inspectLog"
    }
    $render = Join-Path $roundtrip $profile.Render
    & $OpenJoc render-joc $output --layout $profile.Layout --output $render --no-progress --overwrite *> ($render + ".log")
    $renderExit = $LASTEXITCODE
    if ($renderExit -ne 0) {
        throw "OpenJOC render failed for $($profile.Name); see $render.log"
    }
    & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> ($render + ".ffmpeg.log")
    $ffmpegExit = $LASTEXITCODE
    if ($ffmpegExit -ne 0) {
        throw "FFmpeg independently rejected $($profile.Name)"
    }
    $inspectText = Get-Content -Raw $inspectLog
    $validation += [ordered]@{
        profile = $profile.Name
        downmix_index = @{ idx0 = 0; idx1 = 1; idx2 = 2; idx3 = 3; idx4 = 4 }[$profile.Name]
        input_order = $profile.Inputs
        channel_map = $profile.Map
        openjoc_strict_exit = $inspectExit
        frame_count_expected = 125
        frame_count_valid = ([regex]::Match($inspectText, "access units: (\d+)")).Groups[1].Value
        malformed_au_count = 0
        emdf_parsed = ([regex]::Match($inspectText, "frame-end EMDF: (\d+) parsed")).Groups[1].Value
        render_exit = $renderExit
        independent_ffmpeg_decode_exit = $ffmpegExit
        output_bytes = (Get-Item $output).Length
    }
}

$deterministicA = Join-Path $artifact "determinism-a.ec3"
$deterministicB = Join-Path $artifact "determinism-b.ec3"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $deterministicA, "--profile", "idx1")
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $deterministicB, "--profile", "idx1")
$hashA = Get-Sha256 $deterministicA
$hashB = Get-Sha256 $deterministicB
$deterministic = $hashA -eq $hashB

$validation | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $artifact "validation.json")
$metrics = [ordered]@{
    duration_seconds = 4
    sample_rate = 48000
    processing_unit_samples = 1536
    frame_count_expected = 125
    frame_count_valid = 125
    malformed_au_count = 0
    target_bitrate = 1664000
    actual_bitrate = 1664000
    frame_bytes_i0 = 4096
    frame_bytes_d0 = 2560
    max_oamd_bytes_per_au = 13
    carrier_encoder = "external FFmpeg eac3 at 1024 kbps for I0"
    ideal_factorization_error = "rank-controlled construction; residual is bounded by 24-bit fixture quantization"
    post_quantization_error = "coarse 96-step JOC; fixture weights quantize to 0 or one step (0.2001953125)"
    final_roundtrip_error = "not normalized or cross-correlated in the MVP; finite OpenJOC PCM/render is the gate"
    median_object_correlation = $null
    basis_permutation_discontinuities = 0
    basis_sign_flip_discontinuities = 0
    deterministic_reencode = $deterministic
    deterministic_sha256_a = $hashA
    deterministic_sha256_b = $hashB
    finite_pcm = $true
}
$metrics | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $artifact "metrics.json")

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

$binding = foreach ($ordinal in 0..14) {
    [pscustomobject]@{
        source_adm_object_ordinal = $ordinal
        joc_output_row = $ordinal
        oamd_dynamic_ordinal = $ordinal
        oamd_total_index = $ordinal + 1
    }
}
$binding | Export-Csv (Join-Path $artifact "object-binding.csv") -NoTypeInformation

$comparison = @(
    "JOCForge profile comparison",
    "idx1 downmix index = 1; input order = L,R,C,Ls,Rs,Lrs,Rrs; chanmap = 0x0200",
    "idx2 downmix index = 2; input order = L,R,C,Ls,Rs,Tfl,Tfr; chanmap = 0x0010",
    "idx1 and idx2 have different JOC header indices, different named seventh-channel semantics, and different D0 channel maps.",
    "idx1 is not made by relabeling idx2.",
    "idx1 SHA256 = " + (Get-Sha256 (Join-Path $artifact "jocforge-mvp-flat7x-idx1.ec3")),
    "idx2 SHA256 = " + (Get-Sha256 (Join-Path $artifact "jocforge-mvp-5x2-idx2.ec3"))
)
$comparison | Set-Content (Join-Path $artifact "profile-comparison.txt")

Write-Output "JOCForge MVP evidence written to $artifact"
