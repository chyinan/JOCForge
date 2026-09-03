# pattern: Imperative Shell

# pattern: Imperative Shell

param(
    [string]$InputPath = "artifacts/d5-repro/controlled-rate-15.bw64",
    [string]$OutputPath = "artifacts/d5-repro/idx0-1024-regression.ec3"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$input = Join-Path $root $InputPath
$output = Join-Path $root $OutputPath
$inspectLog = "$output.openjoc-inspect.log"
$render = "$output.wav"
$renderLog = "$render.openjoc.log"
$ffmpegLog = "$render.ffmpeg.log"
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$OpenJoc = (Resolve-OpenJocValidator $root).executable
$inputParent = Split-Path -Parent $input
New-Item -ItemType Directory -Force $inputParent | Out-Null

if (!(Test-Path -LiteralPath $input)) {
    & cargo run -p jocforge --quiet -- fixture --seconds 1 --objects 15 $input
    if ($LASTEXITCODE -ne 0) {
        throw "failed to create idx0/1024 regression fixture"
    }
}

& cargo run -p jocforge --quiet -- encode $input $output --profile idx0 --bitrate 1024
if ($LASTEXITCODE -ne 0) {
    throw "idx0/5.X/1024 encode failed"
}

& $OpenJoc inspect $output *> $inspectLog
$inspectExit = $LASTEXITCODE
$inspectText = Get-Content -LiteralPath $inspectLog -Raw
$traversalMatch = [regex]::Match($inspectText, "audio-block carrier traversal errors: (\d+)")
$malformedMatch = [regex]::Match($inspectText, "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed")
$traversalErrors = if ($traversalMatch.Success) { [int]$traversalMatch.Groups[1].Value } else { 0 }
$malformedAus = if ($malformedMatch.Success) { [int]$malformedMatch.Groups[1].Value } else { -1 }

& $OpenJoc render-joc $output --layout 2.0 --output $render --no-progress --overwrite *> $renderLog
$renderExit = $LASTEXITCODE

& ffmpeg -hide_banner -v error -i $render -f null - *> $ffmpegLog
$ffmpegExit = $LASTEXITCODE

$finite = $false
$rawPcm = Join-Path ([IO.Path]::GetTempPath()) ("jocforge-idx0-1024-" + $PID + ".f64le")
try {
    & ffmpeg -hide_banner -v error -y -i $render -c:a pcm_f64le -f f64le $rawPcm
    if ($LASTEXITCODE -eq 0 -and (Test-Path -LiteralPath $rawPcm)) {
        $bytes = [IO.File]::ReadAllBytes($rawPcm)
        $finite = $bytes.Length -gt 0 -and $bytes.Length % 8 -eq 0
        for ($index = 0; $finite -and $index -lt $bytes.Length; $index += 8) {
            $value = [BitConverter]::ToDouble($bytes, $index)
            $finite = -not [double]::IsNaN($value) -and -not [double]::IsInfinity($value)
        }
    }
}
finally {
    if (Test-Path -LiteralPath $rawPcm) { Remove-Item -LiteralPath $rawPcm -Force }
}

$pass = $inspectExit -eq 0 -and
    $malformedAus -eq 0 -and
    $traversalErrors -eq 0 -and
    $renderExit -eq 0 -and
    $ffmpegExit -eq 0 -and
    (Test-Path -LiteralPath $render) -and
    $finite
if (!$pass) {
    throw "idx0/5.X/1024 regression failed: inspect=$inspectExit malformed_aus=$malformedAus traversal=$traversalErrors render=$renderExit ffmpeg=$ffmpegExit finite_pcm=$finite"
}
Write-Output "idx0/5.X/1024 regression PASS: inspect=$inspectExit malformed_aus=$malformedAus traversal=$traversalErrors render=$renderExit ffmpeg=$ffmpegExit finite_pcm=$finite"
