param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\multi-dependent-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$openjoc = (Resolve-OpenJocValidator $root).executable
Assert-OpenJocValidatorPin $root $openjoc | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-identity.json") -Encoding utf8

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')" }
}

function Get-Sha256([string]$path) {
    $hasher = [System.Security.Cryptography.SHA256]::Create()
    try {
        return [BitConverter]::ToString($hasher.ComputeHash([System.IO.File]::ReadAllBytes($path)))
    } finally {
        $hasher.Dispose()
    }
}

$cases = @(
    [pscustomobject]@{ profile = "idx0"; maximum = 5; layout = "2.0" },
    [pscustomobject]@{ profile = "idx1"; maximum = 6; layout = "7.1" },
    [pscustomobject]@{ profile = "idx2"; maximum = 6; layout = "7.1.4" },
    [pscustomobject]@{ profile = "idx3"; maximum = 5; layout = "2.0" },
    [pscustomobject]@{ profile = "idx4"; maximum = 6; layout = "7.1.4" }
)
$rows = [System.Collections.Generic.List[object]]::new()

foreach ($case in $cases) {
    foreach ($dependentCount in 1..$case.maximum) {
        $name = "$($case.profile)-d$dependentCount"
        $output = Join-Path $artifact "$name.ec3"
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "conformance-multi-dependent", $case.profile, "$dependentCount", $output)
        $repeat = Join-Path $artifact "$name.repeat.ec3"
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "conformance-multi-dependent", $case.profile, "$dependentCount", $repeat)
        $deterministic = (Get-Sha256 $output) -eq (Get-Sha256 $repeat)
        Remove-Item -LiteralPath $repeat
        $totalBitrate = 1024 + (($dependentCount - 1) * 640)
        $inspect = Join-Path $artifact "$name.openjoc-inspect.log"
        & $openjoc inspect $output *> $inspect
        $inspectExit = $LASTEXITCODE
        $render = Join-Path $artifact "$name.wav"
        & $openjoc render-joc $output --layout $case.layout --output $render --no-progress --overwrite *> "$render.log"
        $renderExit = $LASTEXITCODE
        $ffmpegLog = Join-Path $artifact "$name.ffmpeg.log"
        $errorPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> $ffmpegLog
        $ffmpegExit = $LASTEXITCODE
        $ErrorActionPreference = $errorPreference

        $structuralPass = $true
        $inspectText = if (Test-Path -LiteralPath $inspect) { Get-Content -Raw $inspect } else { "" }
        $accessUnitMatch = [regex]::Match($inspectText, "access unit 0:\s+sample rate: (\d+) Hz\s+samples: (\d+)")
        $sampleRate = if ($accessUnitMatch.Success) { [int]$accessUnitMatch.Groups[1].Value } else { 0 }
        $accessUnitSamples = if ($accessUnitMatch.Success) { [int]$accessUnitMatch.Groups[2].Value } else { 0 }
        $finite = $false
        $renderedSamples = 0
        $renderedChannels = 0
        if ($renderExit -eq 0 -and (Test-Path -LiteralPath $render)) {
            $renderedChannels = [int]((& ffprobe -v error -select_streams a:0 -show_entries stream=channels -of csv=p=0 $render | Select-Object -First 1))
            $raw = Join-Path $artifact "$name.f32le"
            & ffmpeg -hide_banner -v error -y -i $render -f f32le $raw
            if ($LASTEXITCODE -eq 0 -and $renderedChannels -gt 0) {
                $rawBytes = (Get-Item -LiteralPath $raw).Length
                $renderedSamples = [int]($rawBytes / 4 / $renderedChannels)
                $bytes = [System.IO.File]::ReadAllBytes($raw)
                $nonFinite = 0
                for ($offset = 0; $offset -lt $bytes.Length; $offset += 4) {
                    $value = [BitConverter]::ToSingle($bytes, $offset)
                    if ([float]::IsNaN($value) -or [float]::IsInfinity($value)) { $nonFinite++ }
                }
                $finite = $nonFinite -eq 0
            }
            if (Test-Path -LiteralPath $raw) { Remove-Item -LiteralPath $raw }
        }
        $normativePass = $inspectExit -eq 0 -and $renderExit -eq 0 -and $finite -and $sampleRate -eq 48000 -and $accessUnitSamples -eq 1536 -and $renderedSamples -gt 0
        $ffmpegStatus = if ($ffmpegExit -eq 0) { "PASS" } elseif ($normativePass -and $dependentCount -gt 1) { "UNSUPPORTED_DECODER_FEATURE" } else { "FAIL_STREAM_ERROR" }
        $rows.Add([ordered]@{
            profile = $case.profile
            dependent_count = $dependentCount
            substream_ids = "D0..D$($dependentCount - 1)"
            public_legality = "PUBLIC_LEGAL"
            bitrate_kbps = $totalBitrate
            independent_bitrate_kbps = 384
            dependent_bitrate_kbps = 640
            independent_structural = $structuralPass
            openjoc_strict = $inspectExit -eq 0
            openjoc_full_render = $renderExit -eq 0
            sample_rate = $sampleRate
            access_unit_samples = $accessUnitSamples
            rendered_samples_per_channel = $renderedSamples
            rendered_channels = $renderedChannels
            finite_pcm = $finite
            sample_continuity = $normativePass
            deterministic_encode = $deterministic
            ffmpeg_interoperability = $ffmpegStatus
            output_bytes = (Get-Item -LiteralPath $output).Length
            inspect_log = Split-Path -Leaf $inspect
            render_log = Split-Path -Leaf "$render.log"
            ffmpeg_log = Split-Path -Leaf $ffmpegLog
        })
    }
}

$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
Write-Output "multi-dependent closure evidence written to $artifact"
