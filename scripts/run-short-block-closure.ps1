# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\short-block-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

$partitions = @("1+1+1+1+1+1", "3+3", "2+2+2", "1+2+3")
$profiles = @(
    [pscustomobject]@{ name = "idx0"; layout = "2.0" },
    [pscustomobject]@{ name = "idx1"; layout = "7.1.4" },
    [pscustomobject]@{ name = "idx2"; layout = "7.1.4" },
    [pscustomobject]@{ name = "idx3"; layout = "2.0" },
    [pscustomobject]@{ name = "idx4"; layout = "7.1.4" }
)
$rows = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $profiles) {
    foreach ($partition in $partitions) {
        $safe = $partition.Replace('+', '-')
        $output = Join-Path $artifact ($profile.name + "-" + $safe + ".ec3")
        $inspect = Join-Path $artifact ($profile.name + "-" + $safe + ".inspect.log")
        $render = Join-Path $artifact ($profile.name + "-" + $safe + ".wav")
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "conformance-short", $profile.name, $partition, $output)
    & $openjoc inspect $output *> $inspect
    $inspectExit = $LASTEXITCODE
    if ($inspectExit -ne 0) { throw "OpenJOC rejected $($profile.name) short partition $partition" }
    & $openjoc render-joc $output --layout $profile.layout --output $render --no-progress --overwrite *> ($render + ".log")
    $renderExit = $LASTEXITCODE
    if ($renderExit -ne 0) { throw "OpenJOC failed to render short partition $partition" }
    & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> (Join-Path $artifact ($profile.name + "-" + $safe + ".ffmpeg.log"))
    $ffmpegExit = $LASTEXITCODE
    $inspectText = Get-Content -Raw $inspect
    $rows.Add([ordered]@{
        profile = $profile.name
        partition = $partition
        output_bytes = (Get-Item -LiteralPath $output).Length
        openjoc_inspect = ($inspectExit -eq 0)
        openjoc_render = ($renderExit -eq 0)
        ffmpeg_decode = ($ffmpegExit -eq 0)
        access_units = [int]([regex]::Match($inspectText, "access units: (\d+)").Groups[1].Value)
        samples = [int]([regex]::Match($inspectText, "samples: (\d+)").Groups[1].Value)
        malformed_aus = if ($inspectText -match "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed") { [int]$Matches[1] } else { 0 }
    })
    }
}
$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "short-block closure evidence written to $artifact"
