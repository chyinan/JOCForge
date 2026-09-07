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

$roleCensus = Join-Path $root "artifacts\dependent-channel-role-census.csv"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "dependent-role-census", $roleCensus)

$cases = @(
    [pscustomobject]@{ profile = "idx0"; maximum = 5; layout = "2.0" },
    [pscustomobject]@{ profile = "idx1"; maximum = 6; layout = "7.1" },
    [pscustomobject]@{ profile = "idx2"; maximum = 6; layout = "7.1.4" },
    [pscustomobject]@{ profile = "idx3"; maximum = 5; layout = "2.0" },
    [pscustomobject]@{ profile = "idx4"; maximum = 6; layout = "7.1.4" }
)
$rows = [System.Collections.Generic.List[object]]::new()
$ownershipRows = [System.Collections.Generic.List[object]]::new()

foreach ($case in $cases) {
    foreach ($dependentCount in 1..$case.maximum) {
        $name = "$($case.profile)-d$dependentCount"
        $output = Join-Path $artifact "$name.ec3"
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "conformance-multi-dependent", $case.profile, "$dependentCount", $output)
        $repeat = Join-Path $artifact "$name.repeat.ec3"
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "conformance-multi-dependent", $case.profile, "$dependentCount", $repeat)
        $deterministic = (Get-Sha256 $output) -eq (Get-Sha256 $repeat)
        Remove-Item -LiteralPath $repeat
        $repeatOwnership = [System.IO.Path]::ChangeExtension($repeat, "ownership.json")
        if (Test-Path -LiteralPath $repeatOwnership) { Remove-Item -LiteralPath $repeatOwnership }
        $totalBitrate = 1024 + (($dependentCount - 1) * 640)
        $inspect = Join-Path $artifact "$name.openjoc-inspect.log"
        & $openjoc inspect $output *> $inspect
        $inspectExit = $LASTEXITCODE
        $render = Join-Path $artifact "$name.wav"
        $renderLayout = if ($case.profile -in @("idx0", "idx3")) { "5.1" } else { $case.layout }
        & $openjoc render-joc $output --layout $renderLayout --output $render --no-progress --overwrite *> "$render.log"
        $renderExit = $LASTEXITCODE
        $ffmpegLog = Join-Path $artifact "$name.ffmpeg.log"
        $errorPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> $ffmpegLog
        $ffmpegExit = $LASTEXITCODE
        $ErrorActionPreference = $errorPreference

        $inspectText = if (Test-Path -LiteralPath $inspect) { Get-Content -Raw $inspect } else { "" }
        $accessUnitMatch = [regex]::Match($inspectText, "access unit 0:\s+sample rate: (\d+) Hz\s+samples: (\d+)")
        $sampleRate = if ($accessUnitMatch.Success) { [int]$accessUnitMatch.Groups[1].Value } else { 0 }
        $accessUnitSamples = if ($accessUnitMatch.Success) { [int]$accessUnitMatch.Groups[2].Value } else { 0 }
        $finite = $false
        $renderedSamples = 0
        $renderedChannels = 0
        $channelEnergyPass = $false
        $lfeEnergy = 0.0
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
                $energy = [double[]]::new($renderedChannels)
                for ($sampleIndex = 0; $sampleIndex -lt $renderedSamples; $sampleIndex++) {
                    for ($channelIndex = 0; $channelIndex -lt $renderedChannels; $channelIndex++) {
                        $sampleOffset = ($sampleIndex * $renderedChannels + $channelIndex) * 4
                        $sampleValue = [BitConverter]::ToSingle($bytes, $sampleOffset)
                        $energy[$channelIndex] += [double]$sampleValue * [double]$sampleValue
                    }
                }
                $ownedRenderIndices = switch ($case.profile) {
                    "idx0" { @(0, 1, 2, 4, 5) }
                    "idx1" { @(0, 1, 2, 4, 5, 6, 7) }
                    "idx2" { @(0, 1, 2, 6, 7, 8, 9) }
                    "idx3" { @(0, 1, 2, 4, 5) }
                    "idx4" { @(0, 1, 2, 6, 7, 8, 9) }
                    default { @() }
                }
                $channelEnergyPass = $ownedRenderIndices.Count -gt 0 -and @($ownedRenderIndices | Where-Object { $_ -ge $renderedChannels -or $energy[$_] -le 1e-12 }).Count -eq 0
                if ($renderedChannels -gt 3) { $lfeEnergy = $energy[3] }
            }
            if (Test-Path -LiteralPath $raw) { Remove-Item -LiteralPath $raw }
        }
        $ownershipPath = [System.IO.Path]::ChangeExtension($output, "ownership.json")
        if (!(Test-Path -LiteralPath $ownershipPath)) { throw "ownership report missing for $name" }
        $ownership = Get-Content -Raw -LiteralPath $ownershipPath | ConvertFrom-Json
        if ($ownership.structural_validation -ne $true) { throw "structural validation failed for $name" }
        if ($ownership.capacity_validated -ne $true) { throw "capacity validation failed for $name" }
        if ($ownership.source_causality -ne $true) { throw "source causality failed for $name" }
        $renderLog = if (Test-Path -LiteralPath "$render.log") { Get-Content -Raw "$render.log" } else { "" }
        $lfeMatch = [regex]::Match($renderLog, "LFE count: (\d+)")
        $lfeCount = if ($lfeMatch.Success) { [int]$lfeMatch.Groups[1].Value } else { 0 }
        $expectedChannels = switch ($renderLayout) { "5.1" { 6 } "7.1" { 8 } "7.1.4" { 12 } default { 0 } }
        $expectedRenderedSamples = $accessUnitSamples + 32
        $normativePass = $inspectExit -eq 0 -and $renderExit -eq 0 -and $finite -and $sampleRate -eq 48000 -and $accessUnitSamples -eq 1536 -and $renderedSamples -eq $expectedRenderedSamples -and $renderedChannels -eq $expectedChannels -and $lfeCount -eq 1 -and $channelEnergyPass -and $lfeEnergy -le 1e-12
        $ffmpegStatus = if ($ffmpegExit -eq 0) { "PASS" } elseif ($normativePass -and $dependentCount -gt 1) { "UNSUPPORTED_DECODER_FEATURE" } else { "FAIL_STREAM_ERROR" }
        $rows.Add([ordered]@{
            profile = $case.profile
            lfe_state = "absent"
            dependent_count = $dependentCount
            substream_ids = "D0..D$($dependentCount - 1)"
            public_legality = "PUBLIC_LEGAL"
            semantic_maximum = $case.maximum
            render_layout = $renderLayout
            bitrate_kbps = $totalBitrate
            independent_bitrate_kbps = 384
            dependent_bitrate_kbps = 640
            independent_structural = $ownership.structural_validation
            openjoc_strict = $inspectExit -eq 0
            openjoc_full_render = $renderExit -eq 0
            sample_rate = $sampleRate
            access_unit_samples = $accessUnitSamples
            rendered_samples_per_channel = $renderedSamples
            rendered_channels = $renderedChannels
            rendered_lfe_count = $lfeCount
            lfe_energy = $lfeEnergy
            owned_channel_energy = $channelEnergyPass
            finite_pcm = $finite
            sample_continuity = $normativePass
            deterministic_encode = $deterministic
            structural_validation = $ownership.structural_validation
            capacity_validated = $ownership.capacity_validated
            source_causality = $ownership.source_causality
            ffmpeg_interoperability = $ffmpegStatus
            output_bytes = (Get-Item -LiteralPath $output).Length
            inspect_log = Split-Path -Leaf $inspect
            render_log = Split-Path -Leaf "$render.log"
            ffmpeg_log = Split-Path -Leaf $ffmpegLog
        })

        foreach ($dependent in @($ownership.dependents)) {
            $roles = @($dependent.owned_roles)
            $roleCausality = @($dependent.source_causality_by_role)
            for ($roleIndex = 0; $roleIndex -lt $roles.Count; $roleIndex++) {
                $rolePass = $roleIndex -lt $roleCausality.Count -and [bool]$roleCausality[$roleIndex]
                if (!$rolePass) { throw "source causality failed for $name D$($dependent.id) $($roles[$roleIndex])" }
                $ownershipRows.Add([pscustomobject]@{
                    profile = $ownership.profile
                    lfe_state = $ownership.lfe_state
                    topology = $ownership.topology
                    substream = "D$($dependent.id)"
                    channel_role = $roles[$roleIndex]
                    source_fixture = "synthetic source-derived carrier"
                    ownership_type = if ($dependent.lfe_policy -eq "absent") { "replacement/supplement full-band" } else { $dependent.lfe_policy }
                    chanmap = $dependent.channel_map
                    causality_result = "PASS"
                })
            }
        }
    }
}

foreach ($case in $cases) {
    foreach ($replacement in @($false, $true)) {
        foreach ($dependentCount in 1..$case.maximum) {
            $variant = if ($replacement) { "replacement" } else { "supplement" }
            $name = "$($case.profile)-dependent-lfe-$variant-d$dependentCount"
        $output = Join-Path $artifact "$name.ec3"
        $proofArguments = @("run", "-p", "jocforge", "--quiet", "--", "conformance-dependent-lfe", $case.profile, "$dependentCount", $output)
        if ($replacement) { $proofArguments += "--replacement" }
        Invoke-Checked "cargo" $proofArguments
        $repeat = Join-Path $artifact "$name.repeat.ec3"
        $repeatArguments = @("run", "-p", "jocforge", "--quiet", "--", "conformance-dependent-lfe", $case.profile, "$dependentCount", $repeat)
        if ($replacement) { $repeatArguments += "--replacement" }
        Invoke-Checked "cargo" $repeatArguments
        $deterministic = (Get-Sha256 $output) -eq (Get-Sha256 $repeat)
        Remove-Item -LiteralPath $repeat
        $repeatOwnership = [System.IO.Path]::ChangeExtension($repeat, "ownership.json")
        if (Test-Path -LiteralPath $repeatOwnership) { Remove-Item -LiteralPath $repeatOwnership }
        $totalBitrate = 1024 + (($dependentCount - 1) * 640)
        $inspect = Join-Path $artifact "$name.openjoc-inspect.log"
        & $openjoc inspect $output *> $inspect
        $inspectExit = $LASTEXITCODE
        $render = Join-Path $artifact "$name.wav"
        $renderLayout = if ($case.profile -in @("idx0", "idx3")) { "5.1" } else { $case.layout }
        & $openjoc render-joc $output --layout $renderLayout --output $render --no-progress --overwrite *> "$render.log"
        $renderExit = $LASTEXITCODE
        $ffmpegLog = Join-Path $artifact "$name.ffmpeg.log"
        $errorPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> $ffmpegLog
        $ffmpegExit = $LASTEXITCODE
        $ErrorActionPreference = $errorPreference

        $inspectText = if (Test-Path -LiteralPath $inspect) { Get-Content -Raw $inspect } else { "" }
        $accessUnitMatch = [regex]::Match($inspectText, "access unit 0:\s+sample rate: (\d+) Hz\s+samples: (\d+)")
        $sampleRate = if ($accessUnitMatch.Success) { [int]$accessUnitMatch.Groups[1].Value } else { 0 }
        $accessUnitSamples = if ($accessUnitMatch.Success) { [int]$accessUnitMatch.Groups[2].Value } else { 0 }
        $finite = $false
        $renderedSamples = 0
        $renderedChannels = 0
        $channelEnergyPass = $false
        $lfeEnergy = 0.0
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
                $energy = [double[]]::new($renderedChannels)
                for ($sampleIndex = 0; $sampleIndex -lt $renderedSamples; $sampleIndex++) {
                    for ($channelIndex = 0; $channelIndex -lt $renderedChannels; $channelIndex++) {
                        $sampleOffset = ($sampleIndex * $renderedChannels + $channelIndex) * 4
                        $sampleValue = [BitConverter]::ToSingle($bytes, $sampleOffset)
                        $energy[$channelIndex] += [double]$sampleValue * [double]$sampleValue
                    }
                }
                $ownedRenderIndices = switch ($case.profile) {
                    "idx0" { @(0, 1, 2, 3, 4, 5) }
                    "idx1" { @(0, 1, 2, 3, 4, 6, 7) }
                    "idx2" { @(0, 1, 2, 3, 6, 7, 8, 9) }
                    "idx3" { @(0, 1, 2, 3, 4, 5) }
                    "idx4" { @(0, 1, 2, 3, 6, 7, 8, 9) }
                    default { @() }
                }
                $channelEnergyPass = $ownedRenderIndices.Count -gt 0 -and @($ownedRenderIndices | Where-Object { $_ -ge $renderedChannels -or $energy[$_] -le 1e-12 }).Count -eq 0
                if ($renderedChannels -gt 3) { $lfeEnergy = $energy[3] }
            }
            if (Test-Path -LiteralPath $raw) { Remove-Item -LiteralPath $raw }
        }
        $renderLog = if (Test-Path -LiteralPath "$render.log") { Get-Content -Raw "$render.log" } else { "" }
        $lfeMatch = [regex]::Match($renderLog, "LFE count: (\d+)")
        $lfeCount = if ($lfeMatch.Success) { [int]$lfeMatch.Groups[1].Value } else { 0 }
        $expectedChannels = switch ($renderLayout) { "5.1" { 6 } "7.1" { 8 } "7.1.4" { 12 } default { 0 } }
        $expectedRenderedSamples = $accessUnitSamples + 32
        $baselineRow = @($rows | Where-Object { $_.profile -eq $case.profile -and $_.lfe_state -eq "absent" -and $_.dependent_count -eq $dependentCount }) | Select-Object -First 1
        $baselineLfeEnergy = if ($null -eq $baselineRow) { 0.0 } else { [double]$baselineRow.lfe_energy }
        $normativePass = $inspectExit -eq 0 -and $renderExit -eq 0 -and $finite -and $sampleRate -eq 48000 -and $accessUnitSamples -eq 1536 -and $renderedSamples -eq $expectedRenderedSamples -and $renderedChannels -eq $expectedChannels -and $lfeCount -eq 1 -and $channelEnergyPass -and $lfeEnergy -gt ($baselineLfeEnergy + 1e-12)
        $ffmpegStatus = if ($ffmpegExit -eq 0) { "PASS" } elseif ($normativePass) { "UNSUPPORTED_DECODER_FEATURE" } else { "FAIL_STREAM_ERROR" }
        $ownershipPath = [System.IO.Path]::ChangeExtension($output, "ownership.json")
        if (!(Test-Path -LiteralPath $ownershipPath)) { throw "ownership report missing for $name" }
        $ownership = Get-Content -Raw -LiteralPath $ownershipPath | ConvertFrom-Json
        if ($ownership.structural_validation -ne $true) { throw "structural validation failed for $name" }
        if ($ownership.capacity_validated -ne $true) { throw "capacity validation failed for $name" }
        if ($ownership.source_causality -ne $true) { throw "source causality failed for $name" }
        if (!$normativePass) { throw "dependent-LFE render validation failed for $name" }
        $rows.Add([ordered]@{
            profile = $case.profile
            lfe_state = if ($replacement) { "independent-replacement" } else { "dependent" }
            dependent_count = $dependentCount
            substream_ids = "D0..D$($dependentCount - 1)"
            public_legality = "PUBLIC_LEGAL"
            semantic_maximum = $case.maximum
            render_layout = $renderLayout
            bitrate_kbps = $totalBitrate
            independent_bitrate_kbps = 384
            dependent_bitrate_kbps = 640
            independent_structural = $ownership.structural_validation
            openjoc_strict = $inspectExit -eq 0
            openjoc_full_render = $renderExit -eq 0
            sample_rate = $sampleRate
            access_unit_samples = $accessUnitSamples
            rendered_samples_per_channel = $renderedSamples
            rendered_channels = $renderedChannels
            rendered_lfe_count = $lfeCount
            lfe_energy = $lfeEnergy
            owned_channel_energy = $channelEnergyPass
            finite_pcm = $finite
            sample_continuity = $normativePass
            deterministic_encode = $deterministic
            structural_validation = $ownership.structural_validation
            capacity_validated = $ownership.capacity_validated
            source_causality = $ownership.source_causality
            ffmpeg_interoperability = $ffmpegStatus
            output_bytes = (Get-Item -LiteralPath $output).Length
            inspect_log = Split-Path -Leaf $inspect
            render_log = Split-Path -Leaf "$render.log"
            ffmpeg_log = Split-Path -Leaf $ffmpegLog
        })

        foreach ($dependent in @($ownership.dependents)) {
            $roles = @($dependent.owned_roles)
            $roleCausality = @($dependent.source_causality_by_role)
            for ($roleIndex = 0; $roleIndex -lt $roles.Count; $roleIndex++) {
                $rolePass = $roleIndex -lt $roleCausality.Count -and [bool]$roleCausality[$roleIndex]
                if (!$rolePass) { throw "source causality failed for $name D$($dependent.id) $($roles[$roleIndex])" }
                $ownershipRows.Add([pscustomobject]@{
                    profile = $ownership.profile
                lfe_state = $ownership.lfe_state
                    topology = $ownership.topology
                    substream = "D$($dependent.id)"
                    channel_role = $roles[$roleIndex]
                    source_fixture = "synthetic source-derived carrier"
                    ownership_type = if ($dependent.lfe_policy -eq "absent") { "replacement/supplement full-band" } else { $dependent.lfe_policy }
                    chanmap = $dependent.channel_map
                    causality_result = "PASS"
                })
            }
        }
        if ($ownership.independent_lfe -eq $true) {
            $ownershipRows.Add([pscustomobject]@{
                profile = $ownership.profile
                lfe_state = $ownership.lfe_state
                topology = $ownership.topology
                substream = "I0"
                channel_role = "LFE"
                source_fixture = "synthetic FFmpeg I0 LFE; replaced by D0"
                ownership_type = "independent predecessor"
                chanmap = "n/a"
                causality_result = "REPLACED"
            })
        }
        }
    }
}

$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "validation.json") -Encoding utf8
$ownershipRows | Export-Csv -LiteralPath (Join-Path $artifact "channel-ownership.csv") -NoTypeInformation -Encoding utf8
Write-Output "multi-dependent closure evidence written to $artifact"
