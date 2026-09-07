# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\legacy-core-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable
Assert-OpenJocValidatorPin $root $openjoc | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-identity.json") -Encoding utf8

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

$fixture = Join-Path $artifact "fixture.bw64"
$d0Source = Join-Path $artifact "source-idx1.ec3"
$legacyCore = Join-Path $artifact "legacy-ac3-i0.ec3"
$output = Join-Path $artifact "legacy-ac3-i0-plus-d0.ec3"
$render = Join-Path $artifact "legacy-ac3-i0-plus-d0.wav"
$inspect = Join-Path $artifact "legacy-ac3-i0-plus-d0.openjoc-inspect.log"

Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", $fixture)
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $d0Source, "--profile", "idx1", "--bitrate", "1664")

& ffmpeg -hide_banner -loglevel error -y -f lavfi -i "anoisesrc=color=white:sample_rate=48000:amplitude=0.2" -filter_complex "[0:a]pan=5.1|c0=c0|c1=c0|c2=c0|c3=c0|c4=c0|c5=c0" -t 0.032 -c:a ac3 -b:a 640k -f ac3 $legacyCore
if ($LASTEXITCODE -ne 0) { throw "FFmpeg AC-3 legacy core encoding failed" }
$ac3 = [IO.File]::ReadAllBytes($legacyCore)
$source = [IO.File]::ReadAllBytes($d0Source)
if ($source.Length -lt 6656) { throw "source JOC fixture is shorter than one I0+D0 AU" }
$d0 = $source[4096..6655]
$combined = New-Object byte[] ($ac3.Length + $d0.Length)
[Array]::Copy($ac3, 0, $combined, 0, $ac3.Length)
[Array]::Copy($d0, 0, $combined, $ac3.Length, $d0.Length)
[IO.File]::WriteAllBytes($output, $combined)

& $openjoc inspect $output *> $inspect
if ($LASTEXITCODE -ne 0) { throw "OpenJOC rejected legacy AC-3 I0 plus D0" }
& $openjoc render-joc $output --layout 7.1.4 --output $render --no-progress --overwrite *> ($render + ".log")
if ($LASTEXITCODE -ne 0) { throw "OpenJOC failed to render legacy AC-3 I0 plus D0" }
& ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> (Join-Path $artifact "ffmpeg.log")
$ffmpegExit = $LASTEXITCODE

[ordered]@{
    ac3_core_bytes = $ac3.Length
    d0_bytes = $d0.Length
    combined_bytes = $combined.Length
    openjoc_inspect = $true
    openjoc_render = $true
    ffmpeg_decode_exit = $ffmpegExit
    ffmpeg_decode = ($ffmpegExit -eq 0)
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
if ($ffmpegExit -ne 0) { throw "FFmpeg rejected legacy AC-3 I0 plus D0" }
Write-Output "legacy core closure evidence written to $artifact"
