# pattern: Imperative Shell

$FinalRenderReferenceWindowSamples = 256
$FinalRenderMaximumAlignmentLag = 128
$FinalRenderAlignmentStride = 4

function Read-F64PcmStream([string]$InputPath, [int]$ForcedChannels = 0) {
    if (!(Test-Path -LiteralPath $InputPath)) {
        throw "final-render PCM input does not exist: $InputPath"
    }
    $probeArguments = @("-v", "error")
    if ([IO.Path]::GetExtension($InputPath) -eq ".ec3") { $probeArguments += @("-f", "eac3") }
    $probeArguments += @("-show_entries", "stream=channels", "-of", "default=nw=1:nk=1", $InputPath)
    $channelText = (& ffprobe @probeArguments 2>&1) -join "`n"
    if ($LASTEXITCODE -ne 0) {
        throw "failed to inspect final-render PCM '$InputPath': $channelText"
    }
    $channels = [int]$channelText.Trim()
    if ($ForcedChannels -gt 0) {
        $channels = $ForcedChannels
    }
    if ($channels -le 0) {
        throw "final-render PCM has no channels: $InputPath"
    }
    $rawPath = Join-Path ([IO.Path]::GetTempPath()) ("jocforge-final-render-$PID-" + [IO.Path]::GetRandomFileName() + ".f64")
    try {
        $arguments = @("-hide_banner", "-loglevel", "error", "-y")
        if ([IO.Path]::GetExtension($InputPath) -eq ".ec3") { $arguments += @("-f", "eac3") }
        $arguments += @("-i", $InputPath)
        if ($ForcedChannels -gt 0) {
            $arguments += @("-ac", "$ForcedChannels")
        }
        $arguments += @("-c:a", "pcm_f64le", "-f", "f64le", $rawPath)
        & ffmpeg @arguments
        if ($LASTEXITCODE -ne 0) {
            throw "failed to decode final-render PCM '$InputPath'"
        }
        $bytes = [IO.File]::ReadAllBytes($rawPath)
        if ($bytes.Length % 8 -ne 0 -or $bytes.Length / 8 % $channels -ne 0) {
            throw "final-render PCM has an invalid f64 sample layout: $InputPath"
        }
        $values = New-Object double[] ($bytes.Length / 8)
        for ($index = 0; $index -lt $values.Length; $index++) {
            $values[$index] = [BitConverter]::ToDouble($bytes, $index * 8)
        }
        [pscustomobject]@{
            path = [IO.Path]::GetFullPath($InputPath)
            channels = $channels
            samples = [int]($values.Length / $channels)
            values = $values
        }
    }
    finally {
        if (Test-Path -LiteralPath $rawPath) {
            Remove-Item -LiteralPath $rawPath -Force
        }
    }
}

function Get-ChannelCorrelation(
    [pscustomobject]$Reference,
    [pscustomobject]$Candidate,
    [int]$ReferenceChannel,
    [int]$CandidateChannel,
    [int]$Lag,
    [int]$MaximumSamples = 8192
) {
    $count = [Math]::Min($MaximumSamples, [Math]::Min($Reference.samples, $Candidate.samples - $Lag))
    if ($count -le 1) {
        return 0.0
    }
    $referenceMean = 0.0
    $candidateMean = 0.0
    for ($sample = 0; $sample -lt $count; $sample++) {
        $referenceMean += $Reference.values[$sample * $Reference.channels + $ReferenceChannel]
        $candidateMean += $Candidate.values[($sample + $Lag) * $Candidate.channels + $CandidateChannel]
    }
    $referenceMean /= $count
    $candidateMean /= $count
    $referenceEnergy = 0.0
    $candidateEnergy = 0.0
    $crossEnergy = 0.0
    for ($sample = 0; $sample -lt $count; $sample++) {
        $referenceValue = $Reference.values[$sample * $Reference.channels + $ReferenceChannel] - $referenceMean
        $candidateValue = $Candidate.values[($sample + $Lag) * $Candidate.channels + $CandidateChannel] - $candidateMean
        $referenceEnergy += $referenceValue * $referenceValue
        $candidateEnergy += $candidateValue * $candidateValue
        $crossEnergy += $referenceValue * $candidateValue
    }
    if ($referenceEnergy -le 0.0 -or $candidateEnergy -le 0.0) {
        return 0.0
    }
    $crossEnergy / [Math]::Sqrt($referenceEnergy * $candidateEnergy)
}

function Find-FinalRenderLag(
    [pscustomobject]$Reference,
    [pscustomobject]$Candidate,
    [int]$ReferenceChannel,
    [int]$CandidateChannel
) {
    $bestLag = 0
    $bestCorrelation = -1.0
    for ($lag = 0; $lag -le $FinalRenderMaximumAlignmentLag; $lag += $FinalRenderAlignmentStride) {
        $correlation = [Math]::Abs((Get-ChannelCorrelation $Reference $Candidate $ReferenceChannel $CandidateChannel $lag 4096))
        if ($correlation -gt $bestCorrelation) {
            $bestCorrelation = $correlation
            $bestLag = $lag
        }
    }
    $bestLag
}

function Get-ChannelSpectrum(
    [pscustomobject]$Stream,
    [int]$Channel,
    [int]$Start,
    [int]$Length = $FinalRenderReferenceWindowSamples
) {
    if ($Start -lt 0 -or $Start + $Length -gt $Stream.samples) {
        throw "final-render spectrum window is outside PCM: $($Stream.path)"
    }
    $real = New-Object double[] ([int]($Length / 2) + 1)
    $imaginary = New-Object double[] ([int]($Length / 2) + 1)
    for ($bin = 0; $bin -lt $real.Length; $bin++) {
        for ($sample = 0; $sample -lt $Length; $sample++) {
            $window = 0.5 - 0.5 * [Math]::Cos(2.0 * [Math]::PI * $sample / ($Length - 1))
            $angle = 2.0 * [Math]::PI * $bin * $sample / $Length
            $value = $Stream.values[($Start + $sample) * $Stream.channels + $Channel] * $window
            $real[$bin] += $value * [Math]::Cos($angle)
            $imaginary[$bin] -= $value * [Math]::Sin($angle)
        }
    }
    [pscustomobject]@{ real = $real; imaginary = $imaginary }
}

function Get-PairMagnitudeSpectrum(
    [pscustomobject]$Stream,
    [int]$Start,
    [int]$LeftChannel = 0,
    [int]$RightChannel = 1
) {
    $left = Get-ChannelSpectrum $Stream $LeftChannel $Start
    $right = Get-ChannelSpectrum $Stream $RightChannel $Start
    $magnitude = New-Object double[] $left.real.Length
    for ($bin = 0; $bin -lt $magnitude.Length; $bin++) {
        $magnitude[$bin] = [Math]::Sqrt((
                $left.real[$bin] * $left.real[$bin] +
                $left.imaginary[$bin] * $left.imaginary[$bin] +
                $right.real[$bin] * $right.real[$bin] +
                $right.imaginary[$bin] * $right.imaginary[$bin]
            ) / 2.0)
    }
    $magnitude
}

function Get-BandEnergies([double[]]$Magnitude) {
    $bands = @(0.0, 0.0, 0.0)
    for ($bin = 0; $bin -lt $Magnitude.Length; $bin++) {
        $band = if ($bin -lt 5) { 0 } elseif ($bin -lt 32) { 1 } else { 2 }
        $bands[$band] += $Magnitude[$bin] * $Magnitude[$bin]
    }
    $bands
}

function Get-RelativeSpectrumError([double[]]$Reference, [double[]]$Candidate) {
    $numerator = 0.0
    $denominator = 0.0
    for ($index = 0; $index -lt [Math]::Min($Reference.Length, $Candidate.Length); $index++) {
        $difference = $Reference[$index] - $Candidate[$index]
        $numerator += $difference * $difference
        $denominator += $Reference[$index] * $Reference[$index]
    }
    if ($denominator -le 1.0e-12) { return 0.0 }
    [Math]::Sqrt($numerator / $denominator)
}

function Get-SpectralTiltError([double[]]$Reference, [double[]]$Candidate) {
    $referenceBands = Get-BandEnergies $Reference
    $candidateBands = Get-BandEnergies $Candidate
    $referenceRatio = $referenceBands[2] / [Math]::Max($referenceBands[0], 1.0e-12)
    $candidateRatio = $candidateBands[2] / [Math]::Max($candidateBands[0], 1.0e-12)
    [Math]::Abs([Math]::Log($candidateRatio) - [Math]::Log($referenceRatio))
}

function Get-InterChannelPhase([pscustomobject]$Stream, [int]$Start) {
    $left = Get-ChannelSpectrum $Stream 0 $Start
    $right = Get-ChannelSpectrum $Stream 1 $Start
    $bestBin = 0
    $bestEnergy = -1.0
    for ($bin = 0; $bin -lt $left.real.Length; $bin++) {
        $energy = $left.real[$bin] * $left.real[$bin] + $left.imaginary[$bin] * $left.imaginary[$bin] +
            $right.real[$bin] * $right.real[$bin] + $right.imaginary[$bin] * $right.imaginary[$bin]
        if ($energy -gt $bestEnergy) {
            $bestEnergy = $energy
            $bestBin = $bin
        }
    }
    $leftReal = $left.real[$bestBin]
    $leftImaginary = $left.imaginary[$bestBin]
    $rightReal = $right.real[$bestBin]
    $rightImaginary = $right.imaginary[$bestBin]
    [Math]::Atan2(
        $leftReal * $rightImaginary - $leftImaginary * $rightReal,
        $leftReal * $rightReal + $leftImaginary * $rightImaginary
    )
}

function Get-WrappedPhaseDifference([double]$Left, [double]$Right) {
    $difference = $Left - $Right
    while ($difference -gt [Math]::PI) { $difference -= 2.0 * [Math]::PI }
    while ($difference -lt -[Math]::PI) { $difference += 2.0 * [Math]::PI }
    [Math]::Abs($difference)
}

function Get-TransientPhaseDiscontinuity(
    [pscustomobject]$Reference,
    [pscustomobject]$Candidate,
    [int]$Lag,
    [int]$ChannelCount
) {
    $count = [Math]::Min(8192, [Math]::Min($Reference.samples, $Candidate.samples - $Lag))
    $referencePeak = 0.0
    for ($sample = 0; $sample -lt $count; $sample++) {
        for ($channel = 0; $channel -lt $ChannelCount; $channel++) {
            $referencePeak = [Math]::Max($referencePeak, [Math]::Abs($Reference.values[$sample * $Reference.channels + $channel]))
        }
    }
    $scale = [Math]::Max($referencePeak, 1.0e-6)
    $maximum = 0.0
    for ($sample = 1; $sample -lt $count; $sample++) {
        for ($channel = 0; $channel -lt $ChannelCount; $channel++) {
            $referencePrevious = $Reference.values[($sample - 1) * $Reference.channels + $channel]
            $referenceCurrent = $Reference.values[$sample * $Reference.channels + $channel]
            $candidatePrevious = $Candidate.values[($sample - 1 + $Lag) * $Candidate.channels + $channel]
            $candidateCurrent = $Candidate.values[($sample + $Lag) * $Candidate.channels + $channel]
            $maximum = [Math]::Max($maximum, [Math]::Abs((
                        $candidateCurrent - $candidatePrevious
                    ) - ($referenceCurrent - $referencePrevious)) / $scale)
        }
    }
    $maximum
}

function Measure-FinalRenderEvidence(
    [string]$EncodedPath,
    [string]$RenderedPath,
    [string]$Profile,
    [double]$SourceLeakage,
    [string]$Relationship = "preserved"
) {
    $referenceChannels = if ($Profile -eq "idx0") { 2 } else { 0 }
    $reference = Read-F64PcmStream $EncodedPath $referenceChannels
    $candidate = Read-F64PcmStream $RenderedPath
    $comparableChannels = if ($Profile -eq "idx0") { 2 } else { 3 }
    if ($candidate.channels -lt $comparableChannels -or $reference.channels -lt $comparableChannels) {
        throw "final-render channel layout is too small for $Profile"
    }
    $lag = Find-FinalRenderLag $reference $candidate 0 0
    $correlations = @(
        for ($channel = 0; $channel -lt $comparableChannels; $channel++) {
            Get-ChannelCorrelation $reference $candidate $channel $channel $lag 8192
        }
    )
    $windowStart = if ($Relationship -eq "transient") { 0 } else { 1024 }
    $candidateWindowStart = $windowStart + $lag
    if ($reference.samples -lt $windowStart + $FinalRenderReferenceWindowSamples -or
        $candidate.samples -lt $candidateWindowStart + $FinalRenderReferenceWindowSamples) {
        throw "final-render PCM is too short for spectral evidence"
    }
    $referenceMagnitude = Get-PairMagnitudeSpectrum $reference $windowStart
    $candidateMagnitude = Get-PairMagnitudeSpectrum $candidate $candidateWindowStart
    $spectralError = Get-RelativeSpectrumError $referenceMagnitude $candidateMagnitude
    $spectralTiltError = Get-SpectralTiltError $referenceMagnitude $candidateMagnitude
    $phaseError = Get-WrappedPhaseDifference (
        Get-InterChannelPhase $candidate $candidateWindowStart
    ) (
        Get-InterChannelPhase $reference $windowStart
    )
    $finite = $true
    $peak = 0.0
    foreach ($value in $candidate.values) {
        if ([double]::IsNaN($value) -or [double]::IsInfinity($value)) {
            $finite = $false
        }
        $peak = [Math]::Max($peak, [Math]::Abs($value))
    }
    $activeChannels = if ($Profile -eq "idx0") {
        @(0, 1)
    } elseif ($Profile -eq "idx2") {
        @(0, 1, 2, 4, 5, 6, 7, 8, 9)
    } else {
        @(0, 1, 2, 4, 5, 6, 7)
    }
    $activeChannelSet = [System.Collections.Generic.HashSet[int]]::new()
    foreach ($channel in $activeChannels) { $null = $activeChannelSet.Add($channel) }
    $channelStats = [System.Collections.Generic.List[object]]::new()
    for ($channel = 0; $channel -lt $candidate.channels; $channel++) {
        $channelEnergy = 0.0
        $channelPeak = 0.0
        for ($sample = 0; $sample -lt $candidate.samples; $sample++) {
            $value = $candidate.values[$sample * $candidate.channels + $channel]
            $channelEnergy += $value * $value
            $channelPeak = [Math]::Max($channelPeak, [Math]::Abs($value))
        }
        $channelStats.Add([pscustomobject]@{
            channel = $channel
            active = $activeChannelSet.Contains($channel)
            rms = [Math]::Sqrt($channelEnergy / [Math]::Max(1, $candidate.samples))
            peak = $channelPeak
            finite = $channelPeak -ge 0.0 -and
                -not [double]::IsNaN($channelEnergy) -and -not [double]::IsInfinity($channelEnergy)
        })
    }
    $channelCoverage = $activeChannels.Count -gt 0 -and
        @($activeChannels | Where-Object { $_ -ge $candidate.channels }).Count -eq 0 -and
        @($channelStats | Where-Object {
            ($_.active -and $_.rms -le 1.0e-5) -or $_.finite -ne $true -or $_.peak -gt 0.90
        }).Count -eq 0
    $renderedPhase = Get-InterChannelPhase $candidate $candidateWindowStart
    $referencePhase = Get-InterChannelPhase $reference $windowStart
    $quadratureError = if ($Relationship -eq "quadrature") {
        [Math]::Abs([Math]::Abs($renderedPhase) - [Math]::PI / 2.0)
    } else {
        $phaseError
    }
    [pscustomobject]@{
        stage = "FINAL_DECODED_RENDER"
        profile = $Profile
        finite = $finite
        encoded_path = [IO.Path]::GetFullPath($EncodedPath)
        rendered_path = [IO.Path]::GetFullPath($RenderedPath)
        relationship = $Relationship
        reference_decoder = "FFmpeg independent decode of the same E-AC-3 carrier"
        rendered_channels = $candidate.channels
        reference_channels = $reference.channels
        comparable_channels = $comparableChannels
        rendered_samples = $candidate.samples
        active_render_channels = $activeChannels
        channel_coverage = $channelCoverage
        channel_stats = $channelStats
        alignment_lag_samples = $lag
        channel_correlations = @($correlations | ForEach-Object { [double]$_ })
        phase_coherence = [double](@($correlations | ForEach-Object { [Math]::Abs([double]$_) } | Measure-Object -Minimum).Minimum)
        polarity_preserved = (@($correlations | Where-Object { [double]$_ -lt 0.0 }).Count -eq 0)
        spectral_error = $spectralError
        spectral_tilt_error = $spectralTiltError
        reference_phase_difference = $referencePhase
        rendered_phase_difference = $renderedPhase
        phase_difference_error = $phaseError
        quadrature_phase_error = $quadratureError
        transient_phase_discontinuity = Get-TransientPhaseDiscontinuity $reference $candidate $lag $comparableChannels
        leakage = $SourceLeakage
        peak = $peak
        headroom_margin = 0.90 - $peak
        source_leakage_measurement = "maximum unexplained_cluster_leakage from actual encoded source PCM metrics"
    }
}
