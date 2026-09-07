# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\multi-dependent-closure"
$roundtrip = Join-Path $artifact "roundtrip"
New-Item -ItemType Directory -Force $roundtrip | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable
Assert-OpenJocValidatorPin $root $openjoc | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-identity.json") -Encoding utf8

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

$rows = [System.Collections.Generic.List[object]]::new()
foreach ($count in @(1, 2)) {
    $output = Join-Path $artifact ("idx0-d" + $count + ".ec3")
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "conformance-multi-dependent", "idx0", "$count", $output)
    $inspect = Join-Path $artifact ("idx0-d" + $count + ".openjoc-inspect.log")
    & $openjoc inspect $output *> $inspect
    if ($LASTEXITCODE -ne 0) { throw "OpenJOC strict inspect failed for 5.X D0..D$($count - 1)" }
    $render = Join-Path $roundtrip ("idx0-d" + $count + ".wav")
    & $openjoc render-joc $output --layout 2.0 --output $render --no-progress --overwrite *> ($render + ".log")
    if ($LASTEXITCODE -ne 0) { throw "OpenJOC render failed for D0..D$($count - 1)" }
    & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> (Join-Path $artifact ("idx0-d" + $count + ".ffmpeg.log"))
    if ($LASTEXITCODE -ne 0) { throw "FFmpeg decode failed for 5.X D0..D$($count - 1)" }
    $inspectText = Get-Content -Raw $inspect
    $rows.Add([ordered]@{
        dependent_count = $count
        substream_ids = "D0..D$($count - 1)"
        openjoc_strict = $true
        openjoc_render = $true
        ffmpeg_decode = $true
        output_bytes = (Get-Item -LiteralPath $output).Length
        parsed_access_units = [int]([regex]::Match($inspectText, "access units: (\d+)").Groups[1].Value)
        malformed_aus = [int]([regex]::Match($inspectText, "malformed\D+(\d+)").Groups[1].Value)
        emdf_parsed = [int]([regex]::Match($inspectText, "frame-end EMDF: (\d+) parsed").Groups[1].Value)
    })
}
$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "multi-dependent closure evidence written to $artifact"
