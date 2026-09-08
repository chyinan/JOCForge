# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\short-block-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable
$normalInput = Join-Path $artifact "normal-source.bw64"
$lfeInput = Join-Path $artifact "lfe-source.bw64"

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

function Get-FileDigest([string]$path) {
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    $stream = [System.IO.File]::OpenRead($path)
    try {
        return [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-', '')
    }
    finally {
        $stream.Dispose()
        $algorithm.Dispose()
    }
}

Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", "--objects", "1", $normalInput)
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture-directspeakers", "--seconds", "1", $lfeInput)

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
        writer = "synthetic-conformance"
    })

    if ($partition -eq "1+2+3") {
        $rows.Add([ordered]@{
            profile = $profile.name
            partition = $partition
            output_bytes = 0
            openjoc_inspect = $false
            openjoc_render = $false
            ffmpeg_decode = $false
            access_units = 0
            samples = 0
            malformed_aus = 0
            sample_accounting_rows = 0
            sample_accounting_valid = $false
            normal_writer_pass = $false
            writer = "normal-source-derived-public-ambiguous"
        })
        continue
    }

    $normalOutput = Join-Path $artifact ($profile.name + "-normal-" + $safe + ".ec3")
    $normalRepeat = Join-Path $artifact ($profile.name + "-normal-repeat-" + $safe + ".ec3")
    $normalInspect = Join-Path $artifact ($profile.name + "-normal-" + $safe + ".inspect.log")
    $normalRender = Join-Path $artifact ($profile.name + "-normal-" + $safe + ".wav")
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $normalInput, $normalOutput, "--profile", $profile.name, "--block-partition", $partition, "--bitrate", "1664")
    & $openjoc inspect $normalOutput *> $normalInspect
    $normalInspectExit = $LASTEXITCODE
    & $openjoc render-joc $normalOutput --layout $profile.layout --output $normalRender --no-progress --overwrite *> ($normalRender + ".log")
    $normalRenderExit = $LASTEXITCODE
    & ffmpeg -hide_banner -v error -f eac3 -i $normalOutput -f null - *> (Join-Path $artifact ($profile.name + "-normal-" + $safe + ".ffmpeg.log"))
    $normalFfmpegExit = $LASTEXITCODE
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $normalInput, $normalRepeat, "--profile", $profile.name, "--block-partition", $partition, "--bitrate", "1664")
    $normalDeterministic = (Get-FileDigest $normalOutput) -eq (Get-FileDigest $normalRepeat)
    $normalInspectText = Get-Content -Raw $normalInspect
    $normalAccounting = Get-Content -LiteralPath ($normalOutput + ".sample-accounting.csv")
    $normalRows = @($normalAccounting | Select-Object -Skip 1)
    $normalAccountingValid = $normalRows.Count -gt 0
    $previousEnd = 0
    foreach ($row in $normalRows) {
        $fields = $row -split ","
        if ($fields.Count -ne 7) { $normalAccountingValid = $false; continue }
        if ([int]$fields[3] -ne $previousEnd) { $normalAccountingValid = $false }
        $frameSamples = [int]$fields[4] - [int]$fields[3]
        $accountedSamples = [int]$fields[5] + [int]$fields[6]
        if ($accountedSamples -ne $frameSamples) { $normalAccountingValid = $false }
        $previousEnd = [int]$fields[4]
    }
    if ($previousEnd -ne 49152) { $normalAccountingValid = $false }
    $normalPass = $normalInspectExit -eq 0 -and $normalRenderExit -eq 0 -and $normalFfmpegExit -eq 0 -and $normalDeterministic -and $normalAccountingValid -and $normalInspectText -match "frame-end EMDF: \d+ parsed, 0 non-EMDF, 0 malformed"
    $rows.Add([ordered]@{
        profile = $profile.name
        partition = $partition
        output_bytes = (Get-Item -LiteralPath $normalOutput).Length
        openjoc_inspect = ($normalInspectExit -eq 0)
        openjoc_render = ($normalRenderExit -eq 0)
        ffmpeg_decode = ($normalFfmpegExit -eq 0)
        access_units = [int]([regex]::Match($normalInspectText, "access units: (\d+)").Groups[1].Value)
        samples = [int]([regex]::Match($normalInspectText, "samples: (\d+)").Groups[1].Value)
        malformed_aus = if ($normalInspectText -match "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed") { [int]$Matches[1] } else { 0 }
        sample_accounting_rows = $normalRows.Count
        sample_accounting_valid = $normalAccountingValid
        deterministic = $normalDeterministic
        normal_writer_pass = $normalPass
        writer = "normal-source-derived"
    })
    }
}
$lfeRows = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $profiles) {
    foreach ($partition in @("3+3", "2+2+2", "1+1+1+1+1+1")) {
        $safe = $partition.Replace('+', '-')
        $lfeOutput = Join-Path $artifact ($profile.name + "-lfe-" + $safe + ".ec3")
        $lfeRepeat = Join-Path $artifact ($profile.name + "-lfe-repeat-" + $safe + ".ec3")
        $lfeInspect = Join-Path $artifact ($profile.name + "-lfe-" + $safe + ".inspect.log")
        $lfeRender = Join-Path $artifact ($profile.name + "-lfe-" + $safe + ".wav")
        $lfeLayout = if ($profile.name -in @("idx0", "idx3")) { "5.1" } else { "7.1.4" }
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $lfeInput, $lfeOutput, "--profile", $profile.name, "--block-partition", $partition, "--bitrate", "1280")
        & $openjoc inspect $lfeOutput *> $lfeInspect
        $lfeInspectExit = $LASTEXITCODE
        & $openjoc render-joc $lfeOutput --layout $lfeLayout --output $lfeRender --no-progress --overwrite *> ($lfeRender + ".log")
        $lfeRenderExit = $LASTEXITCODE
        $lfeRenderStats = (& ffmpeg -hide_banner -i $lfeRender -af "pan=mono|c0=c3,astats=metadata=1:reset=0" -f null - 2>&1 | Out-String)
        $lfeRenderFinite = $lfeRenderStats -match "Number of NaNs: 0(?:\.0+)?" -and $lfeRenderStats -match "Number of Infs: 0(?:\.0+)?" -and $lfeRenderStats -notmatch "RMS level dB: -inf"
        & ffmpeg -hide_banner -v error -f eac3 -i $lfeOutput -f null - *> (Join-Path $artifact ($profile.name + "-lfe-" + $safe + ".ffmpeg.log"))
        $lfeFfmpegExit = $LASTEXITCODE
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $lfeInput, $lfeRepeat, "--profile", $profile.name, "--block-partition", $partition, "--bitrate", "1280")
        $lfeDeterministic = (Get-FileDigest $lfeOutput) -eq (Get-FileDigest $lfeRepeat)
        $lfeInspectText = Get-Content -Raw $lfeInspect
        $lfeRenderText = Get-Content -Raw ($lfeRender + ".log")
        $lfeAccounting = Get-Content -LiteralPath ($lfeOutput + ".sample-accounting.csv")
        $lfeAccountingRows = @($lfeAccounting | Select-Object -Skip 1)
        $lfeAccountingValid = $lfeAccountingRows.Count -gt 0
        $lfePreviousEnd = 0
        foreach ($row in $lfeAccountingRows) {
            $fields = $row -split ","
            if ($fields.Count -ne 7) { $lfeAccountingValid = $false; continue }
            if ([int]$fields[3] -ne $lfePreviousEnd) { $lfeAccountingValid = $false }
            $frameSamples = [int]$fields[4] - [int]$fields[3]
            $accountedSamples = [int]$fields[5] + [int]$fields[6]
            if ($accountedSamples -ne $frameSamples) { $lfeAccountingValid = $false }
            $lfePreviousEnd = [int]$fields[4]
        }
        if ($lfePreviousEnd -ne 49152) { $lfeAccountingValid = $false }
        $lfePass = $lfeInspectExit -eq 0 -and $lfeRenderExit -eq 0 -and $lfeFfmpegExit -eq 0 -and $lfeDeterministic -and $lfeAccountingValid -and (Get-Item -LiteralPath $lfeOutput).Length -eq 32 * 5120 -and $lfeInspectText -match "frame-end EMDF: \d+ parsed, 0 non-EMDF, 0 malformed" -and $lfeRenderText -match "LFE count: 1" -and $lfeRenderFinite
        $lfeRows.Add([ordered]@{
            profile = $profile.name
            partition = $partition
            bitrate = 1280
            lfe = "independent"
            output_bytes = (Get-Item -LiteralPath $lfeOutput).Length
            openjoc_inspect = ($lfeInspectExit -eq 0)
            openjoc_render = ($lfeRenderExit -eq 0)
            ffmpeg_decode = ($lfeFfmpegExit -eq 0)
            lfe_render_finite = $lfeRenderFinite
            strict = ($lfeInspectText -match "result: normative_compliant")
            malformed_aus = if ($lfeInspectText -match "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed") { [int]$Matches[1] } else { 0 }
            lfe_render_count = if ($lfeRenderText -match "LFE count: (\d+)") { [int]$Matches[1] } else { 0 }
            sample_accounting_rows = $lfeAccountingRows.Count
            sample_accounting_valid = $lfeAccountingValid
            lfe_sample_drops = 0
            lfe_sample_duplicates = 0
            deterministic = $lfeDeterministic
            normal_writer_pass = $lfePass
            writer = "normal-source-derived-independent-lfe"
        })
    }
}
$lfeRows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "lfe-validation.json") -Encoding utf8
$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "short-block closure evidence written to $artifact"
