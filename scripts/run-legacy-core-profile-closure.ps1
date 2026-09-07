# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\legacy-core-profile-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

$fixture = Join-Path $artifact "fixture.bw64"
$legacyCore = Join-Path $artifact "legacy-ac3-i0.ec3"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", $fixture)
& ffmpeg -hide_banner -loglevel error -y -f lavfi -i "anoisesrc=color=white:sample_rate=48000:amplitude=0.2" -filter_complex "[0:a]pan=5.1|c0=c0|c1=c0|c2=c0|c3=c0|c4=c0|c5=c0" -t 0.032 -c:a ac3 -b:a 640k -f ac3 $legacyCore
if ($LASTEXITCODE -ne 0) { throw "FFmpeg AC-3 core failed" }
$ac3 = [IO.File]::ReadAllBytes($legacyCore)
$profiles = @("idx0", "idx1", "idx2", "idx3", "idx4")
$rows = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $profiles) {
    $source = Join-Path $artifact ("source-" + $profile + ".ec3")
    $output = Join-Path $artifact ("legacy-" + $profile + ".ec3")
    $inspect = Join-Path $artifact ("legacy-" + $profile + ".inspect.log")
    $render = Join-Path $artifact ("legacy-" + $profile + ".wav")
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $source, "--profile", $profile, "--bitrate", "1664")
    $sourceBytes = [IO.File]::ReadAllBytes($source)
    $d0 = $sourceBytes[4096..6655]
    $combined = New-Object byte[] ($ac3.Length + $d0.Length)
    [Array]::Copy($ac3, 0, $combined, 0, $ac3.Length)
    [Array]::Copy($d0, 0, $combined, $ac3.Length, $d0.Length)
    [IO.File]::WriteAllBytes($output, $combined)
    & $openjoc inspect $output *> $inspect
    $inspectExit = $LASTEXITCODE
    & $openjoc render-joc $output --layout 7.1.4 --output $render --no-progress --overwrite *> ($render + ".log")
    $renderExit = $LASTEXITCODE
    & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> (Join-Path $artifact ("legacy-" + $profile + ".ffmpeg.log"))
    $ffmpegExit = $LASTEXITCODE
    $rows.Add([ordered]@{
        profile = $profile
        ac3_core_bytes = $ac3.Length
        d0_bytes = $d0.Length
        openjoc_inspect = ($inspectExit -eq 0)
        openjoc_render = ($renderExit -eq 0)
        ffmpeg_decode = ($ffmpegExit -eq 0)
    })
}
$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "legacy profile closure evidence written to $artifact"
