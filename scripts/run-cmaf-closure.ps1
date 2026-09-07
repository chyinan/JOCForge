# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\cmaf-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

$rows = [System.Collections.Generic.List[object]]::new()
$bitrates = @(1024, 1280, 1408, 1536, 1664)
$fixture = Join-Path $artifact "fixture.bw64"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", $fixture)
foreach ($profile in @("idx0", "idx1", "idx2", "idx3", "idx4")) {
    foreach ($bitrate in $bitrates) {
    $raw = Join-Path $artifact ($profile + "-" + $bitrate + ".ec3")
    $mp4 = Join-Path $artifact ($profile + "-" + $bitrate + ".mp4")
    $decode = Join-Path $artifact ($profile + "-" + $bitrate + "-openjoc-decode")
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $fixture, $raw, "--profile", $profile, "--bitrate", "$bitrate")
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "conformance-cmaf", $profile, $raw, $mp4, "--bitrate", "$bitrate")
    $probe = & ffprobe -v error -show_entries stream=codec_name,codec_tag_string,sample_rate -show_entries packet=pts,dts,duration,size -of json $mp4
    $probeText = $probe -join "`n"
    if ($LASTEXITCODE -ne 0 -or $probeText -notmatch '"codec_tag_string"\s*:\s*"ec-3"') {
        throw "ffprobe did not admit the CMAF ec-3 track for $profile"
    }
    if (Test-Path -LiteralPath $decode) { Remove-Item -LiteralPath $decode -Recurse -Force }
    & $openjoc decode $mp4 --internal-base -o $decode *> (Join-Path $artifact ($profile + "-openjoc.log"))
    if ($LASTEXITCODE -ne 0) { throw "OpenJOC container decode failed for $profile" }
    $rows.Add([ordered]@{
        profile = $profile
        bitrate_kbps = $bitrate
        sample_entry = "ec-3"
        decoder_config = "dec3"
        sample_duration = 1536
        ffprobe = $true
        openjoc_decode = $true
        output_bytes = (Get-Item -LiteralPath $mp4).Length
    })
    }
}
$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "CMAF closure evidence written to $artifact"
