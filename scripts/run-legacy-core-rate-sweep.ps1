# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\legacy-core-rate-sweep"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

$fixture = Join-Path $artifact "fixture.bw64"
$d0Source = Join-Path $artifact "source-idx1.ec3"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", $fixture)
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $d0Source, "--profile", "idx1", "--bitrate", "1664")
$source = [IO.File]::ReadAllBytes($d0Source)
$d0 = $source[4096..6655]
$rates = @(
    [pscustomobject]@{ ac3 = 384; total = 1024 },
    [pscustomobject]@{ ac3 = 448; total = 1088 },
    [pscustomobject]@{ ac3 = 512; total = 1152 },
    [pscustomobject]@{ ac3 = 576; total = 1216 },
    [pscustomobject]@{ ac3 = 640; total = 1280 }
)
$rows = [System.Collections.Generic.List[object]]::new()
foreach ($rate in $rates) {
    $core = Join-Path $artifact ("ac3-" + $rate.ac3 + ".ec3")
    $output = Join-Path $artifact ("legacy-" + $rate.total + ".ec3")
    $inspect = Join-Path $artifact ("legacy-" + $rate.total + ".inspect.log")
    & ffmpeg -hide_banner -loglevel error -y -f lavfi -i "anoisesrc=color=white:sample_rate=48000:amplitude=0.2" -filter_complex "[0:a]pan=5.1|c0=c0|c1=c0|c2=c0|c3=c0|c4=c0|c5=c0" -t 0.032 -c:a ac3 -b:a ("$($rate.ac3)k") -f ac3 $core
    if ($LASTEXITCODE -ne 0) { throw "FFmpeg AC-3 $($rate.ac3) kbps failed" }
    $coreBytes = [IO.File]::ReadAllBytes($core)
    $combined = New-Object byte[] ($coreBytes.Length + $d0.Length)
    [Array]::Copy($coreBytes, 0, $combined, 0, $coreBytes.Length)
    [Array]::Copy($d0, 0, $combined, $coreBytes.Length, $d0.Length)
    [IO.File]::WriteAllBytes($output, $combined)
    & $openjoc inspect $output *> $inspect
    $inspectExit = $LASTEXITCODE
    & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> (Join-Path $artifact ("legacy-" + $rate.total + ".ffmpeg.log"))
    $ffmpegExit = $LASTEXITCODE
    $rows.Add([ordered]@{
        ac3_bitrate_kbps = $rate.ac3
        total_bitrate_kbps = $rate.total
        ac3_frame_bytes = $coreBytes.Length
        d0_frame_bytes = $d0.Length
        openjoc_strict = ($inspectExit -eq 0)
        ffmpeg_decode = ($ffmpegExit -eq 0)
    })
}
$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "legacy core rate sweep written to $artifact"
