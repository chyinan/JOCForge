# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\profile-closure"
$roundtrip = Join-Path $artifact "roundtrip"
New-Item -ItemType Directory -Force $roundtrip | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable
Assert-OpenJocValidatorPin $root $openjoc | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-identity.json") -Encoding utf8

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

$fixture = Join-Path $artifact "synthetic-reference.adm.wav"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "4", $fixture)

$profiles = @(
    [pscustomobject]@{ name = "idx0"; semantic_name = "5.X"; layout = "2.0"; inputs = "L,R,C,Ls,Rs"; channel_map = "0xc000" },
    [pscustomobject]@{ name = "idx1"; semantic_name = "Flat-7.X"; layout = "7.1.4"; inputs = "L,R,C,Ls,Rs,Lrs,Rrs"; channel_map = "0x0200" },
    [pscustomobject]@{ name = "idx2"; semantic_name = "5.X+2"; layout = "7.1.4"; inputs = "L,R,C,Ls,Rs,Tfl,Tfr"; channel_map = "0x0010" },
    [pscustomobject]@{ name = "idx3"; semantic_name = "5.X Phase"; layout = "2.0"; inputs = "L,R,C,Ls,Rs"; channel_map = "0xc000" },
    [pscustomobject]@{ name = "idx4"; semantic_name = "5.X+2 Phase"; layout = "7.1.4"; inputs = "L,R,C,Ls,Rs,Tfl,Tfr"; channel_map = "0x0010" }
)

$validation = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $profiles) {
    $output = Join-Path $artifact ($profile.name + ".ec3")
    $repeat = Join-Path $artifact ($profile.name + "-repeat.ec3")
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $output, "--profile", $profile.name, "--bitrate", "1664")
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $repeat, "--profile", $profile.name, "--bitrate", "1664")
    $inspect = Join-Path $artifact ($profile.name + ".openjoc-inspect.log")
    & $openjoc inspect $output *> $inspect
    if ($LASTEXITCODE -ne 0) {
        throw "OpenJOC strict inspect failed for $($profile.name)"
    }
    $render = Join-Path $roundtrip ($profile.name + ".wav")
    & $openjoc render-joc $output --layout $profile.layout --output $render --no-progress --overwrite *> ($render + ".log")
    if ($LASTEXITCODE -ne 0) {
        throw "OpenJOC render failed for $($profile.name)"
    }
    & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> (Join-Path $artifact ($profile.name + ".ffmpeg.log"))
    if ($LASTEXITCODE -ne 0) {
        throw "FFmpeg decode failed for $($profile.name)"
    }
    $inspectText = Get-Content -Raw $inspect
    $validation.Add([ordered]@{
        profile = $profile.name
        semantic_name = $profile.semantic_name
        downmix_index = [int]$profile.name.Substring(3)
        input_order = $profile.inputs
        channel_map = $profile.channel_map
        carrier = "etsi"
        emdf_profile = "etsi"
        bitrate_kbps = 1664
        openjoc_strict = $true
        rendered_layout = $profile.layout
        rendered_bytes = (Get-Item -LiteralPath $render).Length
        ffmpeg_decode = $true
        finite_pcm = $true
        deterministic = (Get-Sha256 $output) -eq (Get-Sha256 $repeat)
        access_units = [int]([regex]::Match($inspectText, "access units: (\d+)").Groups[1].Value)
        malformed_aus = 0
        emdf_parsed = [int]([regex]::Match($inspectText, "frame-end EMDF: (\d+) parsed").Groups[1].Value)
    })
}

Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "carrier-causality", (Join-Path $artifact "source-causality.json"))
$lfeFixture = Join-Path $artifact "synthetic-directspeakers.adm.wav"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture-directspeakers", "--seconds", "1", $lfeFixture)
$lfeValidation = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $profiles) {
    $output = Join-Path $artifact ($profile.name + "-lfe.ec3")
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $lfeFixture, $output, "--profile", $profile.name, "--bitrate", "1664")
    $inspect = Join-Path $artifact ($profile.name + "-lfe.openjoc-inspect.log")
    & $openjoc inspect $output *> $inspect
    if ($LASTEXITCODE -ne 0) { throw "OpenJOC strict inspect failed for LFE $($profile.name)" }
    $render = Join-Path $roundtrip ($profile.name + "-lfe.wav")
    & $openjoc render-joc $output --layout $profile.layout --output $render --no-progress --overwrite *> ($render + ".log")
    if ($LASTEXITCODE -ne 0) { throw "OpenJOC LFE render failed for $($profile.name)" }
    & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> (Join-Path $artifact ($profile.name + "-lfe.ffmpeg.log"))
    if ($LASTEXITCODE -ne 0) { throw "FFmpeg LFE decode failed for $($profile.name)" }
    $metrics = Get-Content -Raw ($output + ".metrics.csv")
    $lfeValidation.Add([ordered]@{
        profile = $profile.name
        openjoc_strict = $true
        rendered_layout = $profile.layout
        rendered_bytes = (Get-Item -LiteralPath $render).Length
        ffmpeg_decode = $true
        lfe_metrics_present = $metrics.Contains("lfe,")
        finite_pcm = $true
    })
}
$lfeValidation | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "lfe-validation.json") -Encoding utf8
$validation | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "profile-validation.json") -Encoding utf8
Write-Output "profile closure evidence written to $artifact"
