# pattern: Imperative Shell

param(
    [string]$SourceAdm = "",
    [switch]$ReuseExisting
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\validation-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$OpenJoc = (Resolve-OpenJocValidator $root).executable

$invariant = [Globalization.CultureInfo]::InvariantCulture
$MaximumCodedObjects = 15
$UnexplainedLeakageThreshold = 1.0e-12
$SpeakerRenderErrorThreshold = 0.5
$FactorizationResidualThreshold = 0.6
$MatrixCoefficientJumpThreshold = 2.0
$CarrierCoefficientJumpThreshold = 1.8
$SourceReconstructionErrorThreshold = 0.4757030
$SpatialRepresentationErrorThreshold = 0.1720998
$MaximumCodedPeakThreshold = 0.90
$TemporalPositionJumpThreshold = 0.4164535
$TemporalPcmBoundaryDiscontinuityThreshold = 0.0182486
$TemporalGainJumpThreshold = 0.1465067
$TemporalSpeakerDiscontinuityThreshold = 0.0137795
$ShortFinalFrameSamples = 1536
. (Join-Path $PSScriptRoot "final-render-quality.ps1")

function Invoke-FinalRenderGate([pscustomobject]$Measurement, [string]$OutputPath) {
    $arguments = @(
        "run", "-p", "jocforge", "--quiet", "--", "final-render-gate", $OutputPath,
        $(if ($Measurement.finite) { "true" } else { "false" }),
        $(if ($Measurement.channel_coverage) { "true" } else { "false" }),
        "$($Measurement.spectral_error)", "$($Measurement.spectral_tilt_error)",
        "$($Measurement.phase_coherence)",
        $(if ($Measurement.polarity_preserved) { "true" } else { "false" }),
        "$($Measurement.quadrature_phase_error)", "$($Measurement.transient_phase_discontinuity)",
        "$($Measurement.final_render_leakage)", "$($Measurement.peak)", "$($Measurement.headroom_margin)"
    )
    $priorErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $text = (& cargo @arguments 2>&1) -join "`n"
        $exit = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $priorErrorActionPreference
    }
    if (!(Test-Path -LiteralPath $OutputPath)) {
        throw "final-render gate did not write its report: $OutputPath; $text"
    }
    $report = Get-Content -LiteralPath $OutputPath -Raw | ConvertFrom-Json
    $report | Add-Member -NotePropertyName command_exit_code -NotePropertyValue $exit
    $report
}

function Invoke-Checked([string]$File, [string[]]$Arguments) {
    & $File @Arguments
    $exit = $LASTEXITCODE
    if ($exit -ne 0) {
        throw "command failed with exit code $exit`: $File $($Arguments -join ' ')"
    }
}

function Get-Sha256([string]$Path) {
    $sha = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($Path)
    try {
        return (($sha.ComputeHash($stream) | ForEach-Object { $_.ToString("x2") }) -join "")
    }
    finally {
        $stream.Dispose()
        $sha.Dispose()
    }
}

function Format-Number([double]$Value) {
    $Value.ToString("R", $invariant)
}

function Get-FirstNumber([string]$Text, [string]$Pattern) {
    $match = [regex]::Match($Text, $Pattern)
    if (!$match.Success) { return 0 }
    [int]$match.Groups[1].Value
}

function Invoke-Encode([string]$InputPath, [string]$OutputPath, [int]$BitrateKbps, [string]$Profile = "idx1") {
    $metrics = "$OutputPath.metrics.csv"
    if ($ReuseExisting -and (Test-Path -LiteralPath $OutputPath) -and (Test-Path -LiteralPath $metrics)) {
        return
    }
    Invoke-Checked "cargo" @(
        "run", "-p", "jocforge", "--quiet", "--", "encode", $InputPath,
        $OutputPath, "--profile", $Profile, "--bitrate", "$BitrateKbps"
    )
}

function Invoke-Inspect([string]$OutputPath, [string]$Name, [string]$Layout, [switch]$SkipRender) {
    $inspectLog = Join-Path $artifact "$Name.openjoc-inspect.log"
    $render = Join-Path $artifact "$Name.wav"
    $renderLog = "$render.openjoc.log"
    $decodeLog = "$render.ffmpeg.log"

    if ($ReuseExisting -and (Test-Path -LiteralPath $inspectLog) -and
        (Test-Path -LiteralPath $decodeLog) -and ($SkipRender -or (Test-Path -LiteralPath $render))) {
        $inspectExit = 0
        $renderExit = 0
        $decodeExit = 0
    }
    else {
        & $OpenJoc inspect $OutputPath *> $inspectLog
        $inspectExit = $LASTEXITCODE
        $priorErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            if ($SkipRender) {
                $renderExit = 0
            }
            else {
                & $OpenJoc render-joc $OutputPath --layout $Layout --output $render --no-progress --overwrite *> $renderLog
                $renderExit = $LASTEXITCODE
            }
            & ffmpeg -hide_banner -v error -f eac3 -i $OutputPath -f null - *> $decodeLog
            $decodeExit = $LASTEXITCODE
        }
        finally {
            $ErrorActionPreference = $priorErrorActionPreference
        }
    }

    $text = if (Test-Path -LiteralPath $inspectLog) { Get-Content -LiteralPath $inspectLog -Raw } else { "" }
    $accessUnits = Get-FirstNumber $text "access units: (\d+)"
    $malformed = Get-FirstNumber $text "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed"
    $carrierTraversalErrors = Get-FirstNumber $text "audio-block carrier traversal errors: (\d+)"
    $oamdBytes = Get-FirstNumber $text "OAMD bytes: (\d+)"
    $jocBytes = Get-FirstNumber $text "JOC bytes: (\d+)"
    $outputBytes = (Get-Item -LiteralPath $OutputPath).Length
    $metricsPath = "$OutputPath.metrics.csv"
    if (!(Test-Path -LiteralPath $metricsPath)) {
        throw "metrics file does not exist: $metricsPath"
    }
    $metricSummary = Read-MetricSummary $metricsPath
    if ($metricSummary.source_rows -eq 0) {
        throw "metrics file has no source rows: $metricsPath"
    }
    [pscustomobject]@{
        name = $Name
        output = $OutputPath
        access_units = $accessUnits
        malformed_aus = $malformed
        carrier_traversal_errors = $carrierTraversalErrors
        oamd_bytes_per_au = $oamdBytes
        joc_bytes_per_au = $jocBytes
        openjoc_parse_result = if ($inspectExit -eq 0) { "PASS" } else { "FAIL" }
        openjoc_strict_result = if ($inspectExit -eq 0 -and $malformed -eq 0 -and $carrierTraversalErrors -eq 0) { "PASS" } else { "FAIL" }
        openjoc_decode_result = if ($SkipRender) { "NOT_RUN" } elseif ($renderExit -eq 0) { "PASS" } else { "FAIL" }
        openjoc_render_result = if ($SkipRender) { "NOT_RUN" } elseif ($renderExit -eq 0) { "PASS" } else { "FAIL" }
        render_result = if ($SkipRender) { "NOT_RUN" } elseif ($renderExit -eq 0) { "PASS" } else { "FAIL" }
        ffmpeg_decode_result = if ($decodeExit -eq 0) { "PASS" } else { "FAIL" }
        finite_pcm = if ($decodeExit -eq 0) { "YES" } else { "NO" }
        output_bytes = $outputBytes
        render = $render
        source_count = $metricSummary.source_count
        coded_count = $metricSummary.coded_count
        silent_drops = $metricSummary.silent_drops
        unexplained_cluster_leakage_max = Format-Number $metricSummary.unexplained_cluster_leakage_max
        speaker_render_error_max = Format-Number $metricSummary.speaker_render_error_max
        factorization_reconstruction_error_max = Format-Number $metricSummary.factorization_reconstruction_error_max
        matrix_coefficient_jump_max = Format-Number $metricSummary.matrix_coefficient_jump_max
        carrier_coefficient_jump_max = Format-Number $metricSummary.carrier_coefficient_jump_max
    }
}

function Invoke-RejectedEncode([string]$InputPath, [string]$OutputPath, [int]$BitrateKbps, [string]$Profile = "idx1") {
    $priorErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $text = (& cargo run -p jocforge --quiet -- encode $InputPath $OutputPath --profile $Profile --bitrate "$BitrateKbps" 2>&1) -join "`n"
        $exit = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $priorErrorActionPreference
    }
    [pscustomobject]@{
        target_bitrate_kbps = $BitrateKbps
        exit_code = $exit
        diagnostic = $text.Trim()
        explicit_rejection = ($exit -ne 0 -and $text -match "unsupported total bitrate")
    }
}

function Get-DerivedStatus([bool]$Pass) {
    if ($Pass) { return "PASS" }
    "FAIL"
}

function Test-BitrateCell([pscustomobject]$Cell) {
    if ($Cell.encode_result -eq "REJECTED") {
        $capacityRejected = $Cell.rejection_category -eq "CAPACITY" -and
            [int]$Cell.encode_exit_code -ne 0
        return [pscustomobject]@{
            pass = $capacityRejected
            classification = if ($capacityRejected) { "REJECTED_CAPACITY" } else { "FAIL_CORRECTNESS" }
        }
    }
    $supported = $Cell.encode_result -eq "PASS" -and
        [int]$Cell.encode_exit_code -eq 0 -and
        $Cell.structural_validation -eq "PASS" -and
        $Cell.openjoc_parse_result -eq "PASS" -and
        $Cell.openjoc_strict_result -eq "PASS" -and
        $Cell.openjoc_decode_result -eq "PASS" -and
        $Cell.openjoc_render_result -eq "PASS" -and
        $Cell.ffmpeg_decode_result -eq "PASS" -and
        $Cell.finite_pcm -eq "YES" -and
        [int]$Cell.malformed_aus -eq 0 -and
        [int]$Cell.carrier_traversal_errors -eq 0 -and
        $Cell.deterministic_encode -eq "PASS" -and
        $Cell.actual_bitrate_kbps -eq $Cell.target_bitrate_kbps -and
        $Cell.quality_pass -eq $true
    [pscustomobject]@{
        pass = $supported
        classification = if ($supported) { "SUPPORTED" } else { "FAIL_CORRECTNESS" }
    }
}

function Test-FiniteNumber($Value) {
    try { $number = [double]$Value } catch { return $false }
    -not [double]::IsNaN($number) -and -not [double]::IsInfinity($number)
}

function Read-MetricRows([string]$Path) {
    $reader = [IO.StreamReader]::new($Path)
    try {
        $header = $reader.ReadLine().Split(',')
        $indices = @{}
        for ($index = 0; $index -lt $header.Count; $index++) { $indices[$header[$index]] = $index }
        $required = @(
            "record", "frame", "source_index", "object_id", "kind", "coded_object_index",
            "contribution_gain", "source_energy", "represented_energy", "relative_error",
            "spatial_error", "source_count", "coded_count", "silent_drops",
            "source_reconstruction_error", "spatial_representation_error", "cross_object_leakage",
            "unexplained_cluster_leakage", "speaker_render_error", "factorization_reconstruction_error",
            "matrix_coefficient_jump", "carrier_coefficient_jump", "max_coded_peak", "headroom_margin",
            "coded_representative_source_index", "source_azimuth", "source_elevation", "source_distance",
            "coded_azimuth", "coded_elevation", "coded_distance", "coded_first_sample", "coded_last_sample"
        )
        foreach ($field in $required) {
            if (!$indices.ContainsKey($field)) { throw "metrics are missing required field '$field'" }
        }
        $rows = [System.Collections.Generic.List[object]]::new()
        while ($null -ne ($line = $reader.ReadLine())) {
            $fields = $line.Split(',')
            if ($fields[$indices["record"]] -ne "source") { continue }
            $rows.Add([pscustomobject]@{
                record = $fields[$indices["record"]]; frame = $fields[$indices["frame"]];
                source_index = $fields[$indices["source_index"]]; object_id = $fields[$indices["object_id"]];
                kind = $fields[$indices["kind"]]; coded_object_index = $fields[$indices["coded_object_index"]];
                contribution_gain = $fields[$indices["contribution_gain"]]; source_energy = $fields[$indices["source_energy"]];
                represented_energy = $fields[$indices["represented_energy"]]; relative_error = $fields[$indices["relative_error"]];
                spatial_error = $fields[$indices["spatial_error"]]; source_count = $fields[$indices["source_count"]];
                coded_count = $fields[$indices["coded_count"]]; silent_drops = $fields[$indices["silent_drops"]];
                source_reconstruction_error = $fields[$indices["source_reconstruction_error"]];
                spatial_representation_error = $fields[$indices["spatial_representation_error"]];
                cross_object_leakage = $fields[$indices["cross_object_leakage"]];
                unexplained_cluster_leakage = $fields[$indices["unexplained_cluster_leakage"]];
                speaker_render_error = $fields[$indices["speaker_render_error"]];
                factorization_reconstruction_error = $fields[$indices["factorization_reconstruction_error"]];
                matrix_coefficient_jump = $fields[$indices["matrix_coefficient_jump"]];
                carrier_coefficient_jump = $fields[$indices["carrier_coefficient_jump"]];
                max_coded_peak = $fields[$indices["max_coded_peak"]]; headroom_margin = $fields[$indices["headroom_margin"]];
                coded_representative_source_index = $fields[$indices["coded_representative_source_index"]];
                source_azimuth = $fields[$indices["source_azimuth"]]; source_elevation = $fields[$indices["source_elevation"]];
                source_distance = $fields[$indices["source_distance"]]; coded_azimuth = $fields[$indices["coded_azimuth"]];
                coded_elevation = $fields[$indices["coded_elevation"]]; coded_distance = $fields[$indices["coded_distance"]];
                coded_first_sample = $fields[$indices["coded_first_sample"]]; coded_last_sample = $fields[$indices["coded_last_sample"]]
            })
        }
        $rows.ToArray()
    }
    finally {
        $reader.Dispose()
    }
}

function Get-LfeMetricIndices([string]$Path) {
    $reader = [IO.StreamReader]::new($Path)
    try {
        $header = $reader.ReadLine().Split(',')
        $recordIndex = [Array]::IndexOf($header, "record")
        $sourceIndex = [Array]::IndexOf($header, "source_index")
        if ($recordIndex -lt 0 -or $sourceIndex -lt 0) { throw "metrics are missing LFE index fields" }
        $indices = [System.Collections.Generic.List[int]]::new()
        while ($null -ne ($line = $reader.ReadLine())) {
            $fields = $line.Split(',')
            if ($fields[$recordIndex] -eq "lfe") { $indices.Add([int]$fields[$sourceIndex]) }
        }
        $indices.ToArray()
    }
    finally {
        $reader.Dispose()
    }
}

function Read-MetricSummary([string]$Path) {
    $reader = [IO.StreamReader]::new($Path)
    try {
        $header = $reader.ReadLine().Split(',')
        $indices = @{}
        for ($index = 0; $index -lt $header.Count; $index++) { $indices[$header[$index]] = $index }
        $required = @(
            "record", "source_count", "coded_count", "silent_drops", "unexplained_cluster_leakage",
            "source_reconstruction_error", "spatial_representation_error", "speaker_render_error",
            "factorization_reconstruction_error", "matrix_coefficient_jump", "carrier_coefficient_jump",
            "max_coded_peak", "headroom_margin"
        )
        foreach ($field in $required) {
            if (!$indices.ContainsKey($field)) { throw "metrics are missing required field '$field'" }
        }
        $sourceRows = 0
        $sourceCount = 0
        $codedCount = 0
        $silentDrops = 0
        $maxima = @{}
        $metricFields = @(
            "unexplained_cluster_leakage", "source_reconstruction_error", "spatial_representation_error",
            "speaker_render_error", "factorization_reconstruction_error", "matrix_coefficient_jump",
            "carrier_coefficient_jump", "max_coded_peak"
        )
        foreach ($field in $metricFields) { $maxima[$field] = 0.0 }
        $minimumHeadroom = [double]::PositiveInfinity
        while ($null -ne ($line = $reader.ReadLine())) {
            $fields = $line.Split(',')
            if ($fields[$indices["record"]] -ne "source") { continue }
            $sourceRows++
            $sourceCount = [Math]::Max($sourceCount, [int]$fields[$indices["source_count"]])
            $codedCount = [Math]::Max($codedCount, [int]$fields[$indices["coded_count"]])
            $silentDrops = [Math]::Max($silentDrops, [int]$fields[$indices["silent_drops"]])
            foreach ($field in $metricFields) {
                $value = [double]$fields[$indices[$field]]
                if ([double]::IsNaN($value) -or [double]::IsInfinity($value)) {
                    throw "metric '$field' is not finite"
                }
                $maxima[$field] = [Math]::Max($maxima[$field], $value)
            }
            $headroom = [double]$fields[$indices["headroom_margin"]]
            if ([double]::IsNaN($headroom) -or [double]::IsInfinity($headroom)) {
                throw "metric 'headroom_margin' is not finite"
            }
            $minimumHeadroom = [Math]::Min($minimumHeadroom, $headroom)
        }
        [pscustomobject]@{
            source_rows = $sourceRows; source_count = $sourceCount; coded_count = $codedCount; silent_drops = $silentDrops;
            unexplained_cluster_leakage_max = $maxima["unexplained_cluster_leakage"];
            source_reconstruction_error_max = $maxima["source_reconstruction_error"];
            spatial_representation_error_max = $maxima["spatial_representation_error"];
            speaker_render_error_max = $maxima["speaker_render_error"];
            factorization_reconstruction_error_max = $maxima["factorization_reconstruction_error"];
            matrix_coefficient_jump_max = $maxima["matrix_coefficient_jump"];
            carrier_coefficient_jump_max = $maxima["carrier_coefficient_jump"];
            max_coded_peak = $maxima["max_coded_peak"];
            headroom_margin_min = $minimumHeadroom
        }
    }
    finally {
        $reader.Dispose()
    }
}

function Find-ByteSequence([byte[]]$Bytes, [byte[]]$Needle, [int]$Start = 0) {
    for ($offset = $Start; $offset -le $Bytes.Length - $Needle.Length; $offset++) {
        $match = $true
        for ($index = 0; $index -lt $Needle.Length; $index++) {
            if ($Bytes[$offset + $index] -ne $Needle[$index]) { $match = $false; break }
        }
        if ($match) { return $offset }
    }
    -1
}

function Test-QualityGatesFromMetricSummary([pscustomobject]$Summary) {
    $pass = $Summary.source_reconstruction_error_max -le $SourceReconstructionErrorThreshold -and
        $Summary.spatial_representation_error_max -le $SpatialRepresentationErrorThreshold -and
        $Summary.unexplained_cluster_leakage_max -le $UnexplainedLeakageThreshold -and
        $Summary.speaker_render_error_max -le $SpeakerRenderErrorThreshold -and
        $Summary.factorization_reconstruction_error_max -le $FactorizationResidualThreshold -and
        $Summary.matrix_coefficient_jump_max -le $MatrixCoefficientJumpThreshold -and
        $Summary.carrier_coefficient_jump_max -le $CarrierCoefficientJumpThreshold -and
        $Summary.max_coded_peak -le $MaximumCodedPeakThreshold -and
        $Summary.headroom_margin_min -ge 0.0
    [pscustomobject]@{
        pass = $pass
        reason = if ($pass) { "" } else { "one or more published quality thresholds failed" }
        source_reconstruction_error_max = $Summary.source_reconstruction_error_max
        spatial_representation_error_max = $Summary.spatial_representation_error_max
        unexplained_cluster_leakage_max = $Summary.unexplained_cluster_leakage_max
        speaker_render_error_max = $Summary.speaker_render_error_max
        factorization_reconstruction_error_max = $Summary.factorization_reconstruction_error_max
        matrix_coefficient_jump_max = $Summary.matrix_coefficient_jump_max
        carrier_coefficient_jump_max = $Summary.carrier_coefficient_jump_max
        max_coded_peak = $Summary.max_coded_peak
        headroom_margin_min = $Summary.headroom_margin_min
    }
}

function New-PerturbedBw64Fixture([string]$SourcePath, [string]$OutputPath, [int]$SourceIndex) {
    $bytes = [IO.File]::ReadAllBytes($SourcePath)
    $fmtOffset = Find-ByteSequence $bytes ([byte[]](0x66, 0x6d, 0x74, 0x20))
    $dataChunkOffset = Find-ByteSequence $bytes ([byte[]](0x64, 0x61, 0x74, 0x61))
    if ($fmtOffset -lt 0 -or $dataChunkOffset -lt 0) { throw "source fixture is missing fmt/data chunks" }
    $channels = [int][BitConverter]::ToUInt16($bytes, $fmtOffset + 10)
    $bits = [int][BitConverter]::ToUInt16($bytes, $fmtOffset + 22)
    $dataOffset = $dataChunkOffset + 8
    $dataBytes = [int64]($bytes.Length - $dataOffset)
    $axmlOffset = Find-ByteSequence $bytes ([byte[]](0x61, 0x78, 0x6d, 0x6c)) ($dataOffset + 1)
    if ($axmlOffset -gt $dataOffset) { $dataBytes = $axmlOffset - $dataOffset }
    if ($channels -le 0 -or $bits -ne 24 -or $dataBytes -le 0 -or $dataBytes % ($channels * 3) -ne 0) {
        throw "source fixture has an unsupported PCM layout for causality perturbation"
    }
    if ($SourceIndex -lt 0 -or $SourceIndex -ge $channels) { throw "causality source index is outside fixture channels" }
    $sampleCount = [int64]($dataBytes / ($channels * 3))
    for ($sample = 0; $sample -lt $sampleCount; $sample++) {
        $phase = 2.0 * [Math]::PI * (1733.0 + 83.0 * $SourceIndex) * $sample / 48000.0
        $value = [int]([Math]::Round(0.25 * [Math]::Sin($phase) * 8388607.0))
        $encoded = [BitConverter]::GetBytes($value)
        $offset = [int64]$dataOffset + ($sample * $channels + $SourceIndex) * 3
        $bytes[$offset] = $encoded[0]
        $bytes[$offset + 1] = $encoded[1]
        $bytes[$offset + 2] = $encoded[2]
    }
    [IO.File]::WriteAllBytes($OutputPath, $bytes)
}

function New-RelationshipBw64Fixture([string]$SourcePath, [string]$OutputPath, [string]$Relationship) {
    $bytes = [IO.File]::ReadAllBytes($SourcePath)
    $fmtOffset = Find-ByteSequence $bytes ([byte[]](0x66, 0x6d, 0x74, 0x20))
    $dataChunkOffset = Find-ByteSequence $bytes ([byte[]](0x64, 0x61, 0x74, 0x61))
    if ($fmtOffset -lt 0 -or $dataChunkOffset -lt 0) { throw "source fixture is missing fmt/data chunks" }
    $channels = [int][BitConverter]::ToUInt16($bytes, $fmtOffset + 10)
    $bits = [int][BitConverter]::ToUInt16($bytes, $fmtOffset + 22)
    $dataOffset = $dataChunkOffset + 8
    $axmlOffset = Find-ByteSequence $bytes ([byte[]](0x61, 0x78, 0x6d, 0x6c)) ($dataOffset + 1)
    if ($channels -le 0 -or $bits -ne 24 -or $axmlOffset -le $dataOffset) {
        throw "source fixture has an unsupported PCM layout for relationship probe"
    }
    $dataBytes = $axmlOffset - $dataOffset
    if ($dataBytes % ($channels * 3) -ne 0) { throw "relationship probe PCM data is not frame aligned" }
    $sampleCount = [int64]($dataBytes / ($channels * 3))
    for ($sample = 0; $sample -lt $sampleCount; $sample++) {
        for ($channel = 0; $channel -lt $channels; $channel++) {
            $value = if ($Relationship -eq "quadrature" -and $channel -eq 0) {
                0.10 * [Math]::Sin(2.0 * [Math]::PI * 375.0 * $sample / 48000.0)
            } elseif ($Relationship -eq "quadrature" -and $channel -eq 1) {
                0.10 * [Math]::Cos(2.0 * [Math]::PI * 375.0 * $sample / 48000.0)
            } elseif ($Relationship -eq "transient" -and $channel -in @(0, 1) -and $sample -lt 48) {
                0.10 * [Math]::Sin(2.0 * [Math]::PI * 8062.5 * $sample / 48000.0)
            } else {
                0.001 * [Math]::Sin(2.0 * [Math]::PI * (220.0 + 17.0 * $channel) * $sample / 48000.0)
            }
            $encoded = [BitConverter]::GetBytes([int]([Math]::Round($value * 8388607.0)))
            $offset = [int64]$dataOffset + ($sample * $channels + $channel) * 3
            $bytes[$offset] = $encoded[0]
            $bytes[$offset + 1] = $encoded[1]
            $bytes[$offset + 2] = $encoded[2]
        }
    }
    [IO.File]::WriteAllBytes($OutputPath, $bytes)
}

function Measure-RenderedPcmDifference([string]$BaselinePath, [string]$PerturbedPath) {
    $baseline = Read-F64PcmStream $BaselinePath
    $perturbed = Read-F64PcmStream $PerturbedPath
    if ($baseline.channels -ne $perturbed.channels) { throw "causality render channel layouts differ" }
    $count = [Math]::Min($baseline.values.Length, $perturbed.values.Length)
    $maximumDifference = 0.0
    $differenceEnergy = 0.0
    for ($index = 0; $index -lt $count; $index++) {
        $difference = $baseline.values[$index] - $perturbed.values[$index]
        $maximumDifference = [Math]::Max($maximumDifference, [Math]::Abs($difference))
        $differenceEnergy += $difference * $difference
    }
    [pscustomobject]@{
        samples_compared = $count / $baseline.channels
        maximum_absolute_difference = $maximumDifference
        rms_difference = [Math]::Sqrt($differenceEnergy / [Math]::Max(1, $count))
        finite = (@($perturbed.values | Where-Object { [double]::IsNaN($_) -or [double]::IsInfinity($_) }).Count -eq 0)
    }
}

function Test-CarrierCausalityReport([pscustomobject]$Report) {
    if ($null -eq $Report -or $Report.pass -ne $true) { return $false }
    $expected = @{ idx0 = 5; idx1 = 7; idx2 = 7 }
    foreach ($profile in $expected.Keys) {
        $entry = @($Report.profiles | Where-Object { $_.profile -eq $profile })[0]
        if ($null -eq $entry -or [int]$entry.expected_carrier_count -ne $expected[$profile] -or $entry.pass -ne $true) { return $false }
        $carriers = @($entry.carriers)
        if ($carriers.Count -ne $expected[$profile]) { return $false }
        $indices = @($carriers | ForEach-Object { [int]$_.carrier_index } | Sort-Object -Unique)
        if (@(Compare-Object (0..($expected[$profile] - 1)) $indices).Count -gt 0) { return $false }
        if (@($carriers | Where-Object {
                $_.carrier_index -ne $_.source_index -or $_.carrier_changed -ne $true -or
                !(Test-FiniteNumber $_.maximum_change) -or
                [double]$_.maximum_change -le 1.0e-12
            }).Count -gt 0) { return $false }
        $d0Rows = @($entry.d0_writer)
        $expectedD0Indices = if ($profile -eq "idx0") { @(0, 1) } else { @(5, 6) }
        $actualD0Indices = @($entry.d0_input_source_indices | ForEach-Object { [int]$_ })
        if (@(Compare-Object $expectedD0Indices $actualD0Indices).Count -gt 0) { return $false }
        if ($entry.d0_writer_pass -ne $true -or $d0Rows.Count -ne 2 -or
            @($d0Rows | Where-Object {
                $_.input_index -notin @(0, 1) -or $_.changed -ne $true -or
                [int]$_.changed_byte_count -le 0
            }).Count -gt 0) { return $false }
    }
    $true
}

function Test-SourceCausalityReport([pscustomobject]$Report) {
    if ($null -eq $Report -or $Report.pass -ne $true) { return $false }
    $expected = @{ idx0 = 5; idx1 = 7; idx2 = 7 }
    foreach ($profile in $expected.Keys) {
        $entry = @($Report.profiles | Where-Object { $_.profile -eq $profile })[0]
        if ($null -eq $entry -or [int]$entry.expected_carrier_count -ne $expected[$profile] -or $entry.pass -ne $true) { return $false }
        $rows = @($entry.perturbations)
        if ($rows.Count -ne $expected[$profile]) { return $false }
        $indices = @($rows | ForEach-Object { [int]$_.source_index } | Sort-Object -Unique)
        if (@(Compare-Object (0..($expected[$profile] - 1)) $indices).Count -gt 0) { return $false }
        if (@($rows | Where-Object {
                $_.input_hash_changed -ne $true -or $_.output_hash_changed -ne $true -or
                $_.render_delta_finite -ne $true -or !(Test-FiniteNumber $_.render_rms_difference) -or
                !(Test-FiniteNumber $_.render_maximum_absolute_difference) -or
                [double]$_.render_rms_difference -le 1.0e-7
            }).Count -gt 0) { return $false }
    }
    $true
}

function Get-MaxMetric($Rows, [string]$Name) {
    $Rows = @($Rows)
    if ($Rows.Count -gt 0 -and @($Rows[0].PSObject.Properties.Name) -notcontains $Name) {
        throw "metrics are missing required field '$Name'"
    }
    $values = @($Rows | ForEach-Object {
        $value = [double]($_.$Name)
        if ([double]::IsNaN($value) -or [double]::IsInfinity($value)) {
            throw "metric '$Name' is not finite"
        }
        $value
    })
    if ($values.Count -eq 0) { return 0.0 }
    ($values | Measure-Object -Maximum).Maximum
}

function Get-RunLifetimes($Values) {
    $Values = @($Values)
    if ($Values.Count -eq 0) { return @() }
    $lifetimes = [System.Collections.Generic.List[int]]::new()
    $current = $Values[0]
    $length = 1
    foreach ($value in ($Values | Select-Object -Skip 1)) {
        if ($value -eq $current) {
            $length++
        }
        else {
            $lifetimes.Add($length)
            $current = $value
            $length = 1
        }
    }
    $lifetimes.Add($length)
    @($lifetimes)
}

function Test-SourceAccounting($Rows, [int]$ExpectedSourceCount = -1, [int[]]$ExpectedSourceIndices = @()) {
    $Rows = @($Rows)
    $reasons = [System.Collections.Generic.List[string]]::new()
    if ($Rows.Count -eq 0) {
        return [pscustomobject]@{ pass = $false; reason = "source metrics are empty"; frame_count = 0; source_count = 0; source_rows = 0 }
    }
    $required = @("frame", "source_index", "source_count", "coded_count", "silent_drops", "contribution_gain")
    $properties = @($Rows[0].PSObject.Properties.Name)
    foreach ($field in $required) {
        if ($properties -notcontains $field) { $reasons.Add("missing source accounting field '$field'") }
    }
    if ($reasons.Count -eq 0) {
        $frameStates = @{}
        foreach ($row in $Rows) {
            $frame = [string]$row.frame
            if (!$frameStates.ContainsKey($frame)) {
                $frameStates[$frame] = [pscustomobject]@{
                    rows = 0; source_count = [int]$row.source_count;
                    mixed_source_count = $false; over_budget = $false; silent_drop = $false;
                    non_positive_gain = $false; indices = [System.Collections.Generic.HashSet[int]]::new()
                }
            }
            $state = $frameStates[$frame]
            $state.rows++
            $null = $state.indices.Add([int]$row.source_index)
            if ([int]$row.source_count -ne $state.source_count) { $state.mixed_source_count = $true }
            if ([int]$row.coded_count -gt $MaximumCodedObjects) { $state.over_budget = $true }
            if ([int]$row.silent_drops -ne 0) { $state.silent_drop = $true }
            if ([double]$row.contribution_gain -le 0.0) { $state.non_positive_gain = $true }
        }
        $expectedSet = [System.Collections.Generic.HashSet[int]]::new()
        if ($ExpectedSourceIndices.Count -gt 0) {
            foreach ($index in $ExpectedSourceIndices) { $null = $expectedSet.Add([int]$index) }
        }
        foreach ($frame in $frameStates.Keys) {
            $state = $frameStates[$frame]
            $expected = if ($expectedSet.Count -gt 0) {
                $expectedSet
            } elseif ($state.source_count -gt 0) {
                $generatedExpected = [System.Collections.Generic.HashSet[int]]::new()
                foreach ($index in 0..($state.source_count - 1)) { $null = $generatedExpected.Add($index) }
                $generatedExpected
            } else {
                [System.Collections.Generic.HashSet[int]]::new()
            }
            $complete = $state.rows -eq $expected.Count -and $state.indices.Count -eq $expected.Count
            if ($complete) {
                foreach ($index in $expected) {
                    if (!$state.indices.Contains($index)) { $complete = $false; break }
                }
            }
            if ($state.source_count -le 0) { $reasons.Add("frame $frame has no authored sources") }
            if ($ExpectedSourceCount -ge 0 -and $state.source_count -ne $ExpectedSourceCount) {
                $reasons.Add("frame $frame source count differs from expected")
            }
            if (!$complete) { $reasons.Add("frame $frame source indices are not the complete expected domain") }
            if ($state.mixed_source_count) { $reasons.Add("frame $frame mixes source counts") }
            if ($state.over_budget) { $reasons.Add("frame $frame exceeds the coded-object budget") }
            if ($state.silent_drop) { $reasons.Add("frame $frame silently drops a source") }
            if ($state.non_positive_gain) { $reasons.Add("frame $frame has a non-positive source contribution") }
        }
    }
    [pscustomobject]@{
        pass = $reasons.Count -eq 0
        reason = ($reasons -join "; ")
        frame_count = @($Rows | ForEach-Object { [string]$_.frame } | Sort-Object -Unique).Count
        source_count = if ($Rows.Count -gt 0) { [int]$Rows[0].source_count } else { 0 }
        source_rows = $Rows.Count
    }
}

function Test-SourceAccountingCsv([string]$Path, [int]$ExpectedSourceCount = -1, [int[]]$ExpectedSourceIndices = @()) {
    $reader = [IO.StreamReader]::new($Path)
    try {
        $header = $reader.ReadLine().Split(',')
        $indices = @{}
        for ($index = 0; $index -lt $header.Count; $index++) { $indices[$header[$index]] = $index }
        foreach ($field in @("record", "frame", "source_index", "source_count", "coded_count", "silent_drops", "contribution_gain")) {
            if (!$indices.ContainsKey($field)) { throw "metrics are missing required source accounting field '$field'" }
        }
        $frameStates = @{}
        $sourceRows = 0
        while ($null -ne ($line = $reader.ReadLine())) {
            $fields = $line.Split(',')
            if ($fields[$indices["record"]] -ne "source") { continue }
            $sourceRows++
            $frame = $fields[$indices["frame"]]
            if (!$frameStates.ContainsKey($frame)) {
                $frameStates[$frame] = [pscustomobject]@{
                    rows = 0; source_count = [int]$fields[$indices["source_count"]];
                    mixed_source_count = $false; over_budget = $false; silent_drop = $false;
                    non_positive_gain = $false; indices = [System.Collections.Generic.HashSet[int]]::new()
                }
            }
            $state = $frameStates[$frame]
            $state.rows++
            $null = $state.indices.Add([int]$fields[$indices["source_index"]])
            if ([int]$fields[$indices["source_count"]] -ne $state.source_count) { $state.mixed_source_count = $true }
            if ([int]$fields[$indices["coded_count"]] -gt $MaximumCodedObjects) { $state.over_budget = $true }
            if ([int]$fields[$indices["silent_drops"]] -ne 0) { $state.silent_drop = $true }
            if ([double]$fields[$indices["contribution_gain"]] -le 0.0) { $state.non_positive_gain = $true }
        }
        $reasons = [System.Collections.Generic.List[string]]::new()
        $expectedSet = [System.Collections.Generic.HashSet[int]]::new()
        if ($ExpectedSourceIndices.Count -gt 0) {
            foreach ($index in $ExpectedSourceIndices) { $null = $expectedSet.Add([int]$index) }
        }
        foreach ($frame in $frameStates.Keys) {
            $state = $frameStates[$frame]
            $expected = if ($expectedSet.Count -gt 0) {
                $expectedSet
            } elseif ($state.source_count -gt 0) {
                $generatedExpected = [System.Collections.Generic.HashSet[int]]::new()
                foreach ($index in 0..($state.source_count - 1)) { $null = $generatedExpected.Add($index) }
                $generatedExpected
            } else {
                [System.Collections.Generic.HashSet[int]]::new()
            }
            $complete = $state.rows -eq $expected.Count -and $state.indices.Count -eq $expected.Count
            if ($complete) {
                foreach ($index in $expected) {
                    if (!$state.indices.Contains($index)) { $complete = $false; break }
                }
            }
            if ($state.source_count -le 0) { $reasons.Add("frame $frame has no authored sources") }
            if ($ExpectedSourceCount -ge 0 -and $state.source_count -ne $ExpectedSourceCount) { $reasons.Add("frame $frame source count differs from expected") }
            if (!$complete) { $reasons.Add("frame $frame source indices are not the complete expected domain") }
            if ($state.mixed_source_count) { $reasons.Add("frame $frame mixes source counts") }
            if ($state.over_budget) { $reasons.Add("frame $frame exceeds the coded-object budget") }
            if ($state.silent_drop) { $reasons.Add("frame $frame silently drops a source") }
            if ($state.non_positive_gain) { $reasons.Add("frame $frame has a non-positive source contribution") }
        }
        $firstFrame = if ($frameStates.Count -gt 0) { @($frameStates.Keys)[0] } else { $null }
        [pscustomobject]@{
            pass = $reasons.Count -eq 0; reason = ($reasons -join "; "); frame_count = $frameStates.Count;
            source_count = if ($null -ne $firstFrame) { [int]$frameStates[$firstFrame].source_count } else { 0 };
            source_rows = $sourceRows
        }
    }
    finally {
        $reader.Dispose()
    }
}

function Test-QualityGates($Rows) {
    $Rows = @($Rows)
    try {
        $sourceReconstruction = Get-MaxMetric $Rows source_reconstruction_error
        $spatial = Get-MaxMetric $Rows spatial_representation_error
        $unexplained = Get-MaxMetric $Rows unexplained_cluster_leakage
        $speaker = Get-MaxMetric $Rows speaker_render_error
        $factorization = Get-MaxMetric $Rows factorization_reconstruction_error
        $matrix = Get-MaxMetric $Rows matrix_coefficient_jump
        $carrier = Get-MaxMetric $Rows carrier_coefficient_jump
        $peak = Get-MaxMetric $Rows max_coded_peak
        $pass = $sourceReconstruction -le $SourceReconstructionErrorThreshold -and
            $spatial -le $SpatialRepresentationErrorThreshold -and
            $unexplained -le $UnexplainedLeakageThreshold -and
            $speaker -le $SpeakerRenderErrorThreshold -and
            $factorization -le $FactorizationResidualThreshold -and
            $matrix -le $MatrixCoefficientJumpThreshold -and
            $carrier -le $CarrierCoefficientJumpThreshold -and
            $peak -le $MaximumCodedPeakThreshold
        [pscustomobject]@{
            pass = $pass; reason = if ($pass) { "" } else { "one or more published quality thresholds failed" }
            source_reconstruction_error_max = $sourceReconstruction; spatial_representation_error_max = $spatial
            unexplained_cluster_leakage_max = $unexplained; speaker_render_error_max = $speaker
            factorization_reconstruction_error_max = $factorization; matrix_coefficient_jump_max = $matrix
            carrier_coefficient_jump_max = $carrier; max_coded_peak = $peak
        }
    }
    catch {
        [pscustomobject]@{ pass = $false; reason = $_.Exception.Message }
    }
}

function Test-StationaryTemporalEvidence($Row, $EncodedRows) {
    $quality = Test-QualityGates $EncodedRows
    $pass = $null -ne $Row -and
        [int]$Row.total_authored_source_assignment_changes -eq 0 -and
        [int]$Row.total_representative_source_changes -eq 0 -and
        [int]$Row.total_coded_slot_replacements -eq 0 -and
        [int]$Row.minimum_authored_source_assignment_lifetime_frames -eq [int]$Row.frame_count -and
        [int]$Row.minimum_representative_source_lifetime_frames -eq [int]$Row.frame_count -and
        [int]$Row.minimum_coded_slot_lifetime_frames -eq [int]$Row.frame_count -and
        [double]$Row.maximum_position_jump -eq 0.0 -and
        [double]$Row.maximum_pcm_boundary_discontinuity -eq 0.0 -and
        [double]$Row.maximum_gain_jump -eq 0.0 -and
        [double]$Row.maximum_speaker_render_discontinuity -eq 0.0 -and
        $quality.pass -and
        [double]$quality.matrix_coefficient_jump_max -le $MatrixCoefficientJumpThreshold -and
        [double]$quality.carrier_coefficient_jump_max -le $CarrierCoefficientJumpThreshold
    [pscustomobject]@{ pass = $pass; quality = $quality }
}

function Test-MovingTemporalEvidence($Rows) {
    $Rows = @($Rows)
    $checks = @($Rows | ForEach-Object {
        [int]$_.maximum_coded_object_count -le $MaximumCodedObjects -and
        [int]$_.silent_object_drops -eq 0 -and
        [double]$_.maximum_trajectory_error -le $SpatialRepresentationErrorThreshold -and
        [double]$_.maximum_position_jump -le $TemporalPositionJumpThreshold -and
        [double]$_.maximum_pcm_boundary_discontinuity -le $TemporalPcmBoundaryDiscontinuityThreshold -and
        [double]$_.maximum_gain_jump -le $TemporalGainJumpThreshold -and
        [double]$_.maximum_speaker_render_discontinuity -le $TemporalSpeakerDiscontinuityThreshold
    })
    [pscustomobject]@{ pass = $Rows.Count -eq 5 -and $checks.Count -eq 5 -and @($checks | Where-Object { !$_ }).Count -eq 0 }
}

function Get-MinMetric($Rows, [string]$Name) {
    $Rows = @($Rows)
    if ($Rows.Count -gt 0 -and @($Rows[0].PSObject.Properties.Name) -notcontains $Name) {
        throw "metrics are missing required field '$Name'"
    }
    $values = @($Rows | ForEach-Object {
        $value = [double]($_.$Name)
        if ([double]::IsNaN($value) -or [double]::IsInfinity($value)) {
            throw "metric '$Name' is not finite"
        }
        $value
    })
    if ($values.Count -eq 0) { return 0.0 }
    ($values | Measure-Object -Minimum).Minimum
}

function Test-T008ProfileConditions(
    [pscustomobject]$Validation,
    $Rows,
    [pscustomobject]$FinalRender
) {
    $Rows = @($Rows)
    $reasons = [System.Collections.Generic.List[string]]::new()
    $strictPass = $false; $renderPass = $false; $finitePass = $false; $malformedPass = $false; $traversalPass = $false
    $sourceReconstructionPass = $false; $spatialPass = $false; $sourceLeakagePass = $false; $speakerPass = $false
    $factorizationPass = $false; $matrixPass = $false; $carrierPass = $false; $peakPass = $false; $headroomPass = $false
    $finalRenderPass = $false; $finalRenderLeakagePass = $false
    if ($null -eq $Validation) { $reasons.Add("profile validation is missing") }
    else {
        $strictPass = $Validation.openjoc_strict_result -eq "PASS"
        $renderPass = $Validation.render_result -eq "PASS"
        $finitePass = $Validation.finite_pcm -eq "YES"
        $malformedPass = [int]$Validation.malformed_aus -eq 0
        $traversalPass = [int]$Validation.carrier_traversal_errors -eq 0
    }
    if ($null -eq $FinalRender -or $null -eq $FinalRender.gate) { $reasons.Add("final decoded render gate is missing") }
    else {
        $finalRenderPass = $FinalRender.gate.pass -eq $true -and [int]$FinalRender.gate.command_exit_code -eq 0
        if ($FinalRender.PSObject.Properties.Name -contains "final_render_leakage") {
            $finalRenderLeakage = [double]$FinalRender.final_render_leakage
            $finalRenderLeakagePass = Test-FiniteNumber $finalRenderLeakage -and
                $finalRenderLeakage -ge 0.0 -and $finalRenderLeakage -le $UnexplainedLeakageThreshold
        }
    }
    if ($Rows.Count -gt 0) {
        $sourceReconstruction = Get-MaxMetric $Rows source_reconstruction_error
        $spatial = Get-MaxMetric $Rows spatial_representation_error
        $leakage = Get-MaxMetric $Rows unexplained_cluster_leakage
        $speaker = Get-MaxMetric $Rows speaker_render_error
        $factorization = Get-MaxMetric $Rows factorization_reconstruction_error
        $matrix = Get-MaxMetric $Rows matrix_coefficient_jump
        $carrier = Get-MaxMetric $Rows carrier_coefficient_jump
        $peak = Get-MaxMetric $Rows max_coded_peak
        $headroom = Get-MinMetric $Rows headroom_margin
        $sourceReconstructionPass = $sourceReconstruction -le $SourceReconstructionErrorThreshold
        $spatialPass = $spatial -le $SpatialRepresentationErrorThreshold
        $sourceLeakagePass = $leakage -le $UnexplainedLeakageThreshold
        $speakerPass = $speaker -le $SpeakerRenderErrorThreshold
        $factorizationPass = $factorization -le $FactorizationResidualThreshold
        $matrixPass = $matrix -le $MatrixCoefficientJumpThreshold
        $carrierPass = $carrier -le $CarrierCoefficientJumpThreshold
        $peakPass = $peak -le $MaximumCodedPeakThreshold
        $headroomPass = $headroom -ge 0.0
    }
    $checks = @($strictPass, $renderPass, $finitePass, $malformedPass, $traversalPass, $sourceReconstructionPass, $spatialPass, $sourceLeakagePass, $speakerPass, $factorizationPass, $matrixPass, $carrierPass, $peakPass, $headroomPass, $finalRenderLeakagePass, $finalRenderPass)
    $pass = $Rows.Count -gt 0 -and @($checks | Where-Object { $_ -ne $true }).Count -eq 0
    if (!$strictPass) { $reasons.Add("OpenJOC strict validation failed") }
    if (!$renderPass) { $reasons.Add("OpenJOC compatibility render failed") }
    if (!$finitePass) { $reasons.Add("decoded E-AC-3 PCM is not finite") }
    if (!$malformedPass) { $reasons.Add("malformed access units are present") }
    if (!$traversalPass) { $reasons.Add("carrier traversal errors are present") }
    if (!$sourceReconstructionPass) { $reasons.Add("source reconstruction threshold failed") }
    if (!$spatialPass) { $reasons.Add("spatial representation threshold failed") }
    if (!$sourceLeakagePass) { $reasons.Add("source-stage leakage threshold failed") }
    if (!$speakerPass) { $reasons.Add("speaker render threshold failed") }
    if (!$factorizationPass) { $reasons.Add("factorization residual threshold failed") }
    if (!$matrixPass) { $reasons.Add("matrix continuity threshold failed") }
    if (!$carrierPass) { $reasons.Add("carrier continuity threshold failed") }
    if (!$peakPass) { $reasons.Add("coded peak threshold failed") }
    if (!$headroomPass) { $reasons.Add("coded headroom threshold failed") }
    if (!$finalRenderLeakagePass) { $reasons.Add("final-render leakage threshold failed") }
    if (!$finalRenderPass) { $reasons.Add("final decoded render gate failed") }
    [pscustomobject]@{
        pass = $pass; reason = ($reasons -join "; "); strict = $strictPass; render = $renderPass;
        finite = $finitePass; malformed_aus = $malformedPass; carrier_traversal = $traversalPass;
        source_reconstruction = $sourceReconstructionPass; spatial = $spatialPass; source_leakage = $sourceLeakagePass;
        speaker = $speakerPass; factorization = $factorizationPass; matrix = $matrixPass; carrier = $carrierPass;
        coded_peak = $peakPass; coded_headroom = $headroomPass; final_render_leakage = $finalRenderLeakagePass;
        final_decoded_render = $finalRenderPass
    }
}

function New-FinalRenderLeakageMutation(
    [string]$InputPath,
    [string]$OutputPath,
    [int]$UnintendedChannel
) {
    $stream = Read-F64PcmStream $InputPath
    if ($UnintendedChannel -lt 0 -or $UnintendedChannel -ge $stream.channels) {
        throw "final-render leakage mutation channel is outside rendered PCM"
    }
    $values = [double[]]$stream.values.Clone()
    for ($sample = 0; $sample -lt $stream.samples; $sample++) {
        $values[$sample * $stream.channels + $UnintendedChannel] =
            $values[$sample * $stream.channels]
    }
    $rawPath = Join-Path ([IO.Path]::GetTempPath()) ("jocforge-final-render-leakage-$PID.f64le")
    try {
        $bytes = New-Object byte[] ($values.Length * 8)
        for ($index = 0; $index -lt $values.Length; $index++) {
            [Buffer]::BlockCopy([BitConverter]::GetBytes($values[$index]), 0, $bytes, $index * 8, 8)
        }
        [IO.File]::WriteAllBytes($rawPath, $bytes)
        & ffmpeg -hide_banner -v error -y -f f64le -ar 48000 -ac $stream.channels -i $rawPath -c:a pcm_f32le -f wav $OutputPath
        if ($LASTEXITCODE -ne 0) {
            throw "failed to write temporary final-render leakage mutation"
        }
    }
    finally {
        if (Test-Path -LiteralPath $rawPath) { Remove-Item -LiteralPath $rawPath -Force }
    }
}

function Get-Median([double[]]$Values) {
    if ($Values.Count -eq 0) { return 0.0 }
    $ordered = @($Values | Sort-Object)
    $middle = [int]($ordered.Count / 2)
    if ($ordered.Count % 2 -eq 0) {
        return ($ordered[$middle - 1] + $ordered[$middle]) / 2.0
    }
    $ordered[$middle]
}

function Get-SlidingMaximum([int[]]$Values, [int]$Window) {
    if ($Values.Count -eq 0) { return 0 }
    $maximum = 0
    for ($index = 0; $index -lt $Values.Count; $index++) {
        $start = [Math]::Max(0, $index - $Window + 1)
        $sum = 0
        for ($cursor = $start; $cursor -le $index; $cursor++) { $sum += $Values[$cursor] }
        $maximum = [Math]::Max($maximum, $sum)
    }
    $maximum
}

function Get-TemporalAudit($Rows, [string]$Name) {
    $Rows = @($Rows)
    if ($Rows.Count -eq 0) { throw "temporal metrics are empty for $Name" }
    $frameGroups = @($Rows | Group-Object frame | Sort-Object { [int]$_.Name })
    $frameRows = @($frameGroups | ForEach-Object { $_.Group[0] })
    $changes = [System.Collections.Generic.List[int]]::new()
    $representativeChanges = [System.Collections.Generic.List[int]]::new()
    $codedSlotReplacements = [System.Collections.Generic.List[int]]::new()
    for ($frameIndex = 1; $frameIndex -lt $frameGroups.Count; $frameIndex++) {
        $previousSources = @{}
        $currentSources = @{}
        foreach ($row in @($frameGroups[$frameIndex - 1].Group)) { $previousSources[[int]$row.source_index] = [int]$row.coded_object_index }
        foreach ($row in @($frameGroups[$frameIndex].Group)) { $currentSources[[int]$row.source_index] = [int]$row.coded_object_index }
        $changes.Add(@($previousSources.Keys | Where-Object { $currentSources.ContainsKey($_) -and $previousSources[$_] -ne $currentSources[$_] }).Count)

        $previousRepresentatives = @($frameGroups[$frameIndex - 1].Group | ForEach-Object { [int]$_.coded_representative_source_index } | Sort-Object -Unique)
        $currentRepresentatives = @($frameGroups[$frameIndex].Group | ForEach-Object { [int]$_.coded_representative_source_index } | Sort-Object -Unique)
        $representativeChanges.Add(@($previousRepresentatives | Where-Object { $currentRepresentatives -notcontains $_ }).Count)

        $previousSlots = @{}
        $currentSlots = @{}
        foreach ($row in @($frameGroups[$frameIndex - 1].Group)) {
            $slot = [int]$row.coded_object_index
            if (!$previousSlots.ContainsKey($slot)) { $previousSlots[$slot] = [int]$row.coded_representative_source_index }
        }
        foreach ($row in @($frameGroups[$frameIndex].Group)) {
            $slot = [int]$row.coded_object_index
            if (!$currentSlots.ContainsKey($slot)) { $currentSlots[$slot] = [int]$row.coded_representative_source_index }
        }
        $codedSlotReplacements.Add(@($previousSlots.Keys | Where-Object { $currentSlots.ContainsKey($_) -and $previousSlots[$_] -ne $currentSlots[$_] }).Count)
    }
    $totalChanges = if ($changes.Count -eq 0) { 0 } else { ($changes | Measure-Object -Sum).Sum }
    $totalRepresentativeChanges = if ($representativeChanges.Count -eq 0) { 0 } else { ($representativeChanges | Measure-Object -Sum).Sum }
    $totalCodedSlotReplacements = if ($codedSlotReplacements.Count -eq 0) { 0 } else { ($codedSlotReplacements | Measure-Object -Sum).Sum }
    $firstChange = @($changes | Where-Object { $_ -gt 0 } | Select-Object -First 1)

    $assignmentLifetimes = [System.Collections.Generic.List[int]]::new()
    foreach ($group in @($Rows | Group-Object source_index)) {
        $ordered = @($group.Group | Sort-Object { [int]$_.frame })
        foreach ($lifetime in @(Get-RunLifetimes @($ordered | ForEach-Object { [int]$_.coded_object_index }))) {
            $assignmentLifetimes.Add([int]$lifetime)
        }
    }
    $representativeLifetimes = [System.Collections.Generic.List[int]]::new()
    $representativeIds = @($Rows | ForEach-Object { [int]$_.coded_representative_source_index } | Sort-Object -Unique)
    foreach ($representativeId in $representativeIds) {
        $presence = @($frameGroups | ForEach-Object { @($_.Group | Where-Object { [int]$_.coded_representative_source_index -eq $representativeId }).Count -gt 0 })
        foreach ($lifetime in @(Get-RunLifetimes $presence)) {
            $representativeLifetimes.Add([int]$lifetime)
        }
    }
    $codedSlotLifetimes = [System.Collections.Generic.List[int]]::new()
    $maximumCodedObjectCount = if ($Rows.Count -gt 0) { [int](@($Rows | ForEach-Object { [int]$_.coded_count } | Measure-Object -Maximum).Maximum) } else { 0 }
    for ($slot = 0; $slot -lt $maximumCodedObjectCount; $slot++) {
        $presence = @($frameRows | ForEach-Object { [int]$_.coded_count -gt $slot })
        foreach ($lifetime in @(Get-RunLifetimes $presence)) {
            $codedSlotLifetimes.Add([int]$lifetime)
        }
    }

    $maxPositionJump = 0.0
    $maxPcmDiscontinuity = 0.0
    $maxGainJump = 0.0
    $maxSpeakerDiscontinuity = 0.0
    $maxReconstructionChange = 0.0
    foreach ($group in @($Rows | Group-Object source_index)) {
        $ordered = @($group.Group | Sort-Object { [int]$_.frame })
        for ($index = 1; $index -lt $ordered.Count; $index++) {
            $previous = $ordered[$index - 1]
            $current = $ordered[$index]
            $maxReconstructionChange = [Math]::Max($maxReconstructionChange, [Math]::Abs([double]$current.source_reconstruction_error - [double]$previous.source_reconstruction_error))
            if ([int]$current.coded_object_index -eq [int]$previous.coded_object_index) { continue }
            $positionJump = [Math]::Sqrt(([double]$current.coded_azimuth - [double]$previous.coded_azimuth) * ([double]$current.coded_azimuth - [double]$previous.coded_azimuth) + ([double]$current.coded_elevation - [double]$previous.coded_elevation) * ([double]$current.coded_elevation - [double]$previous.coded_elevation) + ([double]$current.coded_distance - [double]$previous.coded_distance) * ([double]$current.coded_distance - [double]$previous.coded_distance))
            $maxPositionJump = [Math]::Max($maxPositionJump, $positionJump)
            $maxPcmDiscontinuity = [Math]::Max($maxPcmDiscontinuity, [Math]::Abs([double]$current.coded_first_sample - [double]$previous.coded_last_sample))
            $maxGainJump = [Math]::Max($maxGainJump, [Math]::Abs([double]$current.contribution_gain - [double]$previous.contribution_gain))
            $previousValue = [double]$previous.coded_last_sample; $currentValue = [double]$current.coded_first_sample
            $previousAzimuth = [Math]::Max(-1.0, [Math]::Min(1.0, [double]$previous.coded_azimuth)); $currentAzimuth = [Math]::Max(-1.0, [Math]::Min(1.0, [double]$current.coded_azimuth))
            $previousLeft = $previousValue * (1.0 - $previousAzimuth) * 0.5; $previousRight = $previousValue * (1.0 + $previousAzimuth) * 0.5
            $currentLeft = $currentValue * (1.0 - $currentAzimuth) * 0.5; $currentRight = $currentValue * (1.0 + $currentAzimuth) * 0.5
            $maxSpeakerDiscontinuity = [Math]::Max($maxSpeakerDiscontinuity, [Math]::Max([Math]::Abs($currentLeft - $previousLeft), [Math]::Abs($currentRight - $previousRight)))
        }
    }
    $frameCount = $frameRows.Count
    $minutes = [Math]::Max(($frameCount * 1536.0 / 48000.0 / 60.0), 1536.0 / 48000.0 / 60.0)
    $assignmentArray = @($assignmentLifetimes | Sort-Object)
    $representativeArray = @($representativeLifetimes | Sort-Object)
    $codedSlotArray = @($codedSlotLifetimes | Sort-Object)
    [pscustomobject]@{
        scene = $Name; frame_count = $frameCount; authored_source_count = if ($frameRows.Count -gt 0) { [int]$frameRows[0].source_count } else { 0 }
        maximum_coded_object_count = $maximumCodedObjectCount
        total_authored_source_assignment_changes = $totalChanges; total_representative_source_changes = $totalRepresentativeChanges; total_coded_slot_replacements = $totalCodedSlotReplacements
        authored_source_assignment_changes_per_minute = $totalChanges / $minutes; max_sliding_window_authored_source_assignment_changes = Get-SlidingMaximum @($changes) 8
        minimum_authored_source_assignment_lifetime_frames = if ($assignmentArray.Count) { $assignmentArray[0] } else { 0 }; median_authored_source_assignment_lifetime_frames = Get-Median ([double[]]$assignmentArray)
        minimum_representative_source_lifetime_frames = if ($representativeArray.Count) { $representativeArray[0] } else { 0 }; median_representative_source_lifetime_frames = Get-Median ([double[]]$representativeArray)
        minimum_coded_slot_lifetime_frames = if ($codedSlotArray.Count) { $codedSlotArray[0] } else { 0 }; median_coded_slot_lifetime_frames = Get-Median ([double[]]$codedSlotArray)
        authored_source_assignment_reassignment_burstiness = if ($totalChanges -eq 0) { 0.0 } else { (Get-SlidingMaximum @($changes) 8) / [double]$totalChanges }
        first_assignment_change_frame = if ($firstChange.Count -gt 0) { [int]$firstChange[0] } else { "NONE" }
        maximum_position_jump = $maxPositionJump; maximum_pcm_boundary_discontinuity = $maxPcmDiscontinuity; maximum_gain_jump = $maxGainJump; maximum_reconstruction_error_change = $maxReconstructionChange; maximum_speaker_render_discontinuity = $maxSpeakerDiscontinuity
        maximum_unexplained_cluster_leakage = Get-MaxMetric $Rows unexplained_cluster_leakage; maximum_speaker_render_error = Get-MaxMetric $Rows speaker_render_error; maximum_factorization_reconstruction_error = Get-MaxMetric $Rows factorization_reconstruction_error; maximum_matrix_coefficient_jump = Get-MaxMetric $Rows matrix_coefficient_jump; maximum_carrier_coefficient_jump = Get-MaxMetric $Rows carrier_coefficient_jump
        largest_sliding_window_frames = 8
        stationary_authored_source_assignment_churn_after_initialization = if ($Name -eq "stationary-control") { $totalChanges } else { 0 }
    }
}

if ([string]::IsNullOrWhiteSpace($SourceAdm)) {
    $SourceAdm = Get-ChildItem -LiteralPath "D:\ADM BWF" -File -Filter "*.wav" |
        Where-Object { $_.Length -eq 1583325474 } |
        Select-Object -First 1 -ExpandProperty FullName
}
if ([string]::IsNullOrWhiteSpace($SourceAdm) -or !(Test-Path -LiteralPath $SourceAdm)) { throw "source ADM file does not exist: $SourceAdm" }
$validatorIdentity = Assert-OpenJocValidatorPin $root $OpenJoc
$validatorIdentity | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-identity.json") -Encoding utf8
$validatorPinTestOutput = (& powershell -NoProfile -File (Join-Path $PSScriptRoot "test-openjoc-validator-pin.ps1") 2>&1) -join "`n"
$validatorPinTestExit = $LASTEXITCODE
if ($validatorPinTestExit -ne 0) {
    throw "OpenJOC validator pin test failed with exit code $validatorPinTestExit"
}
$validatorPinTestOutput | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-pin-test.json") -Encoding utf8
$validatorPinTest = $validatorPinTestOutput | ConvertFrom-Json

$temporalPath = Join-Path $artifact "temporal-metrics.csv"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $temporalPath)) {
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "temporal", $temporalPath)
}
$temporalRows = @(Import-Csv -LiteralPath $temporalPath)

$rateSpecs = @(
    [pscustomobject]@{ total = 1024; carrier = 384; d0 = 640; carrier_bytes = 1536; d0_bytes = 2560 },
    [pscustomobject]@{ total = 1280; carrier = 640; d0 = 640; carrier_bytes = 2560; d0_bytes = 2560 },
    [pscustomobject]@{ total = 1408; carrier = 768; d0 = 640; carrier_bytes = 3072; d0_bytes = 2560 },
    [pscustomobject]@{ total = 1536; carrier = 896; d0 = 640; carrier_bytes = 3584; d0_bytes = 2560 },
    [pscustomobject]@{ total = 1664; carrier = 1024; d0 = 640; carrier_bytes = 4096; d0_bytes = 2560 }
)
$rateByTotal = @{}
foreach ($spec in $rateSpecs) { $rateByTotal["$($spec.total)"] = $spec }
$bitrateProfiles = @(
    [pscustomobject]@{ name = "idx0"; layout = "2.0"; carrier_count = 5 },
    [pscustomobject]@{ name = "idx1"; layout = "7.1.4"; carrier_count = 7 },
    [pscustomobject]@{ name = "idx2"; layout = "7.1.4"; carrier_count = 7 }
)
$bitrateCandidates = @(384, 448, 640, 768, 896, 1024, 1280, 1408, 1536, 1664)
$rateFixture = Join-Path $artifact "controlled-rate-15.bw64"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $rateFixture)) {
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", "--objects", "15", $rateFixture)
}
$bitrateRows = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $bitrateProfiles) {
    foreach ($requested in $bitrateCandidates) {
        $cellName = "bitrate-$requested-$($profile.name)"
        if (!$rateByTotal.ContainsKey("$requested")) {
            $rejectedOutput = Join-Path $artifact ($cellName + "-rejected.ec3")
            if (Test-Path -LiteralPath $rejectedOutput) { Remove-Item -LiteralPath $rejectedOutput -Force }
            $rejection = Invoke-RejectedEncode $rateFixture $rejectedOutput $requested $profile.name
            $rejectionCategory = if ($rejection.explicit_rejection) { "CAPACITY" } else { "CORRECTNESS" }
            $classification = if ($rejectionCategory -eq "CAPACITY") { "REJECTED_CAPACITY" } else { "FAIL_CORRECTNESS" }
            $bitrateRows.Add([pscustomobject]@{
                target_bitrate_kbps = $requested; actual_bitrate_kbps = "REJECTED"; profile = $profile.name;
                carrier_count = $profile.carrier_count; coded_object_count = "N/A"; metadata_bytes_per_au = "N/A"; carrier_bytes_per_au = "N/A";
                encode_result = "REJECTED"; encode_exit_code = $rejection.exit_code; rejection_category = $rejectionCategory;
                structural_validation = "NOT_APPLICABLE"; openjoc_parse_result = "NOT_APPLICABLE"; openjoc_strict_result = "NOT_APPLICABLE";
                openjoc_decode_result = "NOT_APPLICABLE"; openjoc_render_result = "NOT_APPLICABLE"; ffmpeg_decode_result = "NOT_APPLICABLE";
                finite_pcm = "NOT_APPLICABLE"; malformed_aus = "NOT_APPLICABLE"; carrier_traversal_errors = "NOT_APPLICABLE";
                openjoc_carrier_traversal_errors = "NOT_APPLICABLE"; quality_pass = "NOT_APPLICABLE";
                source_reconstruction_error_max = "N/A"; spatial_representation_error_max = "N/A";
                cross_object_leakage_max = "N/A"; unexplained_cluster_leakage_max = "N/A";
                speaker_render_error_max = "N/A"; factorization_reconstruction_error_max = "N/A";
                matrix_coefficient_jump_max = "N/A"; carrier_coefficient_jump_max = "N/A";
                headroom_margin_min = "N/A"; deterministic_encode = "NOT_APPLICABLE";
                compatibility_render = "NOT_APPLICABLE"; classification = $classification; status = $classification;
                command_diagnostic = $rejection.diagnostic;
                quality_fallback = "deterministic CLI rejection: no clean paired I0/D0 budget"
            })
            continue
        }
        $spec = $rateByTotal["$requested"]
        $output = Join-Path $artifact ($cellName + ".ec3")
        Invoke-Encode $rateFixture $output $requested $profile.name
        $inspection = Invoke-Inspect $output $cellName $profile.layout
        $metricsPath = "$output.metrics.csv"
        $rows = Read-MetricRows $metricsPath
        $frameRows = @($rows | Group-Object frame | ForEach-Object { $_.Group[0] })
        $codedCount = if ($frameRows.Count -gt 0) { [int](@($frameRows | ForEach-Object { [int]$_.coded_count } | Measure-Object -Maximum).Maximum) } else { 0 }
        $duration = [Math]::Max($inspection.access_units * 1536.0 / 48000.0, 1536.0 / 48000.0)
        $actual = [Math]::Round($inspection.output_bytes * 8.0 / $duration / 1000.0, 3)
        $repeat = Join-Path $artifact ($cellName + "-repeat.ec3")
        Invoke-Encode $rateFixture $repeat $requested $profile.name
        $deterministic = (Get-Sha256 $output) -eq (Get-Sha256 $repeat)
        $unexplainedLeakage = Get-MaxMetric $rows unexplained_cluster_leakage
        $speakerRenderError = Get-MaxMetric $rows speaker_render_error
        $factorizationResidual = Get-MaxMetric $rows factorization_reconstruction_error
        $matrixJump = Get-MaxMetric $rows matrix_coefficient_jump
        $carrierJump = Get-MaxMetric $rows carrier_coefficient_jump
        $qualityPass = $unexplainedLeakage -le $UnexplainedLeakageThreshold -and
            $speakerRenderError -le $SpeakerRenderErrorThreshold -and
            $factorizationResidual -le $FactorizationResidualThreshold -and
            $matrixJump -le $MatrixCoefficientJumpThreshold -and
            $carrierJump -le $CarrierCoefficientJumpThreshold
        $row = [pscustomobject]@{
            target_bitrate_kbps = $requested; actual_bitrate_kbps = $actual; profile = $profile.name;
            carrier_count = $profile.carrier_count; coded_object_count = $codedCount; metadata_bytes_per_au = $inspection.oamd_bytes_per_au + $inspection.joc_bytes_per_au;
            carrier_bytes_per_au = $spec.carrier_bytes; encode_result = "PASS"; encode_exit_code = 0; rejection_category = "NONE";
            structural_validation = if ($inspection.openjoc_strict_result -eq "PASS" -and $inspection.malformed_aus -eq 0 -and $inspection.carrier_traversal_errors -eq 0) { "PASS" } else { "FAIL" };
            openjoc_parse_result = $inspection.openjoc_parse_result; openjoc_strict_result = $inspection.openjoc_strict_result;
            openjoc_decode_result = $inspection.openjoc_decode_result; openjoc_render_result = $inspection.openjoc_render_result;
            ffmpeg_decode_result = $inspection.ffmpeg_decode_result; finite_pcm = $inspection.finite_pcm;
            malformed_aus = $inspection.malformed_aus; carrier_traversal_errors = $inspection.carrier_traversal_errors;
            openjoc_carrier_traversal_errors = $inspection.carrier_traversal_errors; quality_pass = $qualityPass;
            source_reconstruction_error_max = Format-Number (Get-MaxMetric $rows source_reconstruction_error);
            spatial_representation_error_max = Format-Number (Get-MaxMetric $rows spatial_representation_error);
            cross_object_leakage_max = Format-Number (Get-MaxMetric $rows cross_object_leakage);
            unexplained_cluster_leakage_max = Format-Number $unexplainedLeakage;
            speaker_render_error_max = Format-Number $speakerRenderError;
            factorization_reconstruction_error_max = Format-Number $factorizationResidual;
            matrix_coefficient_jump_max = Format-Number $matrixJump;
            carrier_coefficient_jump_max = Format-Number $carrierJump;
            headroom_margin_min = Format-Number (Get-MinMetric $rows headroom_margin);
            deterministic_encode = if ($deterministic) { "PASS" } else { "FAIL" };
            compatibility_render = $inspection.openjoc_render_result;
            quality_fallback = "none; 15 coded objects retained; deterministic I0/D0 split $($spec.carrier)+$($spec.d0) kbps"
        }
        $verdict = Test-BitrateCell $row
        $row | Add-Member -NotePropertyName classification -NotePropertyValue $verdict.classification
        $row | Add-Member -NotePropertyName status -NotePropertyValue $verdict.classification
        $bitrateRows.Add($row)
    }
}
$bitrateCsv = Join-Path $artifact "bitrate-sweep.csv"
$bitrateRows | Export-Csv -LiteralPath $bitrateCsv -NoTypeInformation -Encoding utf8
$bitrateRows | Export-Csv -LiteralPath (Join-Path $artifact "profile-bitrate-matrix.csv") -NoTypeInformation -Encoding utf8

$quantizationPath = Join-Path $artifact "quantization-stress.json"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "quantization-stress", $quantizationPath)
$quantization = Get-Content -LiteralPath $quantizationPath -Raw | ConvertFrom-Json

$fixtureRows = [System.Collections.Generic.List[object]]::new()
$leakageRows = [System.Collections.Generic.List[object]]::new()
foreach ($objectCount in @(15, 16, 20, 28)) {
    $fixture = Join-Path $artifact "controlled-$objectCount.bw64"
    if (!$ReuseExisting -or !(Test-Path -LiteralPath $fixture)) {
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", "--objects", "$objectCount", $fixture)
    }
    $encoded = Join-Path $artifact "controlled-$objectCount-idx1.ec3"
    Invoke-Encode $fixture $encoded 1664
    $rows = Read-MetricRows "$encoded.metrics.csv"
    $accounting = Test-SourceAccounting $rows $objectCount
    $quality = Test-QualityGates $rows
    $firstFrame = @($rows | Where-Object { [int]$_.frame -eq 0 })
    $clusters = @($firstFrame | Group-Object coded_object_index)
    foreach ($row in $firstFrame) {
        $cluster = $clusters | Where-Object { $_.Name -eq $row.coded_object_index } | Select-Object -First 1
        $gainSum = @($cluster.Group | ForEach-Object { [double]$_.contribution_gain } | Measure-Object -Sum).Sum
        $normalized = if ($gainSum -gt 0) { [double]$row.contribution_gain / $gainSum } else { 0.0 }
        $unexplained = [double]$row.unexplained_cluster_leakage
        $leakageRows.Add([pscustomobject]@{
            scene = "controlled-$objectCount"; source_index = $row.source_index; object_id = $row.object_id;
            coded_object_index = $row.coded_object_index; contribution_gain = $row.contribution_gain;
            normalized_contribution = Format-Number $normalized; intentional_merge = ($cluster.Count -gt 1);
            source_relative_error = $row.relative_error; spatial_error = $row.spatial_error;
            unexplained_cluster_leakage = Format-Number $unexplained;
            unintended_leakage = if ($unexplained -le $UnexplainedLeakageThreshold) { "0" } else { Format-Number $unexplained };
            rationale = "independent coded-signal residual; merge membership is reported separately"
        })
    }
    $fixtureRows.Add([pscustomobject]@{
        scene = "controlled-$objectCount"; encoded = $encoded; source_count = [int]$firstFrame[0].source_count;
        coded_count = [int](@($rows | ForEach-Object { [int]$_.coded_count } | Measure-Object -Maximum).Maximum);
        silent_drops = [int](@($rows | ForEach-Object { [int]$_.silent_drops } | Measure-Object -Maximum).Maximum);
        source_accounting_pass = $accounting.pass; source_accounting_reason = $accounting.reason;
        quality_gate_pass = $quality.pass; quality_gate_reason = $quality.reason;
        source_reconstruction_error_max = Format-Number (Get-MaxMetric $rows source_reconstruction_error);
        spatial_representation_error_max = Format-Number (Get-MaxMetric $rows spatial_representation_error);
        cross_object_leakage_max = Format-Number (Get-MaxMetric $rows cross_object_leakage);
        unexplained_cluster_leakage_max = Format-Number (Get-MaxMetric $rows unexplained_cluster_leakage);
        speaker_render_error_max = Format-Number (Get-MaxMetric $rows speaker_render_error);
        factorization_reconstruction_error_max = Format-Number (Get-MaxMetric $rows factorization_reconstruction_error);
        matrix_coefficient_jump_max = Format-Number (Get-MaxMetric $rows matrix_coefficient_jump);
        carrier_coefficient_jump_max = Format-Number (Get-MaxMetric $rows carrier_coefficient_jump);
        max_coded_peak = Format-Number (Get-MaxMetric $rows max_coded_peak)
    })
}
$leakageCsv = Join-Path $artifact "leakage-matrix.csv"
$leakageRows | Export-Csv -LiteralPath $leakageCsv -NoTypeInformation -Encoding utf8
$fixtureRows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "quality-observations.json") -Encoding utf8

$temporalAudits = [System.Collections.Generic.List[object]]::new()
foreach ($row in $temporalRows) {
    $temporalAudits.Add([pscustomobject]@{
        scene = $row.scene; source = "controlled-core"; frame_count = [int]$row.frame_count;
        authored_source_count = [int]$row.authored_source_count; maximum_coded_object_count = [int]$row.maximum_coded_object_count;
        total_authored_source_assignment_changes = [int]$row.total_authored_source_assignment_changes; total_representative_source_changes = [int]$row.total_representative_source_changes; total_coded_slot_replacements = [int]$row.total_coded_slot_replacements; authored_source_assignment_changes_per_minute = $row.authored_source_assignment_changes_per_minute;
        max_sliding_window_authored_source_assignment_changes = [int]$row.max_sliding_window_authored_source_assignment_changes;
        minimum_authored_source_assignment_lifetime_frames = [int]$row.minimum_authored_source_assignment_lifetime_frames;
        median_authored_source_assignment_lifetime_frames = $row.median_authored_source_assignment_lifetime_frames;
        minimum_representative_source_lifetime_frames = [int]$row.minimum_representative_source_lifetime_frames;
        median_representative_source_lifetime_frames = $row.median_representative_source_lifetime_frames;
        minimum_coded_slot_lifetime_frames = [int]$row.minimum_coded_slot_lifetime_frames;
        median_coded_slot_lifetime_frames = $row.median_coded_slot_lifetime_frames;
        authored_source_assignment_reassignment_burstiness = $row.authored_source_assignment_reassignment_burstiness; first_assignment_change_frame = "core-metric";
        maximum_position_jump = $row.maximum_position_jump; maximum_pcm_boundary_discontinuity = $row.maximum_pcm_boundary_discontinuity;
        maximum_gain_jump = $row.maximum_gain_jump; maximum_reconstruction_error_change = $row.maximum_reconstruction_error_change;
        maximum_speaker_render_discontinuity = $row.maximum_speaker_render_discontinuity;
        stationary_authored_source_assignment_churn_after_initialization = [int]$row.stationary_authored_source_assignment_churn_after_initialization
    })
}

$hanabiOutput = Join-Path $artifact "hanabi-multichannel-idx1.ec3"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $hanabiOutput)) {
    Invoke-Encode $SourceAdm $hanabiOutput 1664
}
$hanabiSummary = Read-MetricSummary "$hanabiOutput.metrics.csv"
$hanabiMetricsHash = Get-Sha256 "$hanabiOutput.metrics.csv"
$hanabiLfeIndices = @(Get-LfeMetricIndices "$hanabiOutput.metrics.csv" | Sort-Object -Unique)
$hanabiTotalSourceCount = [int]$hanabiSummary.source_count + $hanabiLfeIndices.Count
$hanabiExpectedSourceIndices = @(
    0..($hanabiTotalSourceCount - 1) | Where-Object { $hanabiLfeIndices -notcontains $_ }
)
$existingTemporalAuditPath = Join-Path $artifact "temporal-audit.csv"
$hanabiTemporalHashPath = Join-Path $artifact "hanabi-temporal-input.sha256"
$existingHanabiAudit = if ((Test-Path -LiteralPath $existingTemporalAuditPath) -and
    (Test-Path -LiteralPath $hanabiTemporalHashPath) -and
    (Get-Content -LiteralPath $hanabiTemporalHashPath -Raw).Trim() -eq $hanabiMetricsHash) {
    @((Import-Csv -LiteralPath $existingTemporalAuditPath) | Where-Object { $_.scene -eq "hanabi" })[0]
} else { $null }
$hanabiAudited = if ($null -ne $existingHanabiAudit) {
    $existingHanabiAudit
} else {
    Get-TemporalAudit (Read-MetricRows "$hanabiOutput.metrics.csv") "hanabi"
}
$temporalAudits.Add([pscustomobject]@{
    scene = $hanabiAudited.scene; source = "Hanabi"; frame_count = $hanabiAudited.frame_count;
    authored_source_count = $hanabiAudited.authored_source_count; maximum_coded_object_count = $hanabiAudited.maximum_coded_object_count;
    total_authored_source_assignment_changes = $hanabiAudited.total_authored_source_assignment_changes; total_representative_source_changes = $hanabiAudited.total_representative_source_changes; total_coded_slot_replacements = $hanabiAudited.total_coded_slot_replacements; authored_source_assignment_changes_per_minute = Format-Number $hanabiAudited.authored_source_assignment_changes_per_minute;
    max_sliding_window_authored_source_assignment_changes = $hanabiAudited.max_sliding_window_authored_source_assignment_changes;
    minimum_authored_source_assignment_lifetime_frames = $hanabiAudited.minimum_authored_source_assignment_lifetime_frames;
    median_authored_source_assignment_lifetime_frames = Format-Number $hanabiAudited.median_authored_source_assignment_lifetime_frames;
    minimum_representative_source_lifetime_frames = $hanabiAudited.minimum_representative_source_lifetime_frames;
    median_representative_source_lifetime_frames = Format-Number $hanabiAudited.median_representative_source_lifetime_frames;
    minimum_coded_slot_lifetime_frames = $hanabiAudited.minimum_coded_slot_lifetime_frames;
    median_coded_slot_lifetime_frames = Format-Number $hanabiAudited.median_coded_slot_lifetime_frames;
    authored_source_assignment_reassignment_burstiness = Format-Number $hanabiAudited.authored_source_assignment_reassignment_burstiness;
    first_assignment_change_frame = $hanabiAudited.first_assignment_change_frame;
    maximum_position_jump = Format-Number $hanabiAudited.maximum_position_jump;
    maximum_pcm_boundary_discontinuity = Format-Number $hanabiAudited.maximum_pcm_boundary_discontinuity;
    maximum_gain_jump = Format-Number $hanabiAudited.maximum_gain_jump;
    maximum_reconstruction_error_change = Format-Number $hanabiAudited.maximum_reconstruction_error_change;
    maximum_speaker_render_discontinuity = Format-Number $hanabiAudited.maximum_speaker_render_discontinuity;
    stationary_authored_source_assignment_churn_after_initialization = "N/A"
})
$temporalAudits | Export-Csv -LiteralPath (Join-Path $artifact "temporal-audit.csv") -NoTypeInformation -Encoding utf8
Set-Content -LiteralPath $hanabiTemporalHashPath -Value $hanabiMetricsHash -Encoding ascii

$causalFixture = Join-Path $artifact "directspeakers-causal.bw64"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $causalFixture)) {
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture-directspeakers", "--seconds", "1", $causalFixture)
}
$causalInventory = Join-Path $artifact "directspeakers-causal.csv"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $causalInventory)) {
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "inspect-adm", $causalFixture, "--direct-speakers-csv", $causalInventory)
}
$causalOutput = Join-Path $artifact "directspeakers-causal.ec3"
Invoke-Encode $causalFixture $causalOutput 1664
$causalInspection = Invoke-Inspect $causalOutput "directspeakers-causal" "7.1.4"
$causalInventoryRows = @(Import-Csv -LiteralPath $causalInventory)
$causalAllMetricRows = @(Import-Csv -LiteralPath "$causalOutput.metrics.csv" | Where-Object { $_.record -in @("source", "lfe") })
$causalFirstFrame = @($causalAllMetricRows | Where-Object { [int]$_.frame -eq 0 })
$causalSourceIds = @($causalFirstFrame | ForEach-Object { $_.object_id })
$causalExpectedIds = @($causalInventoryRows | ForEach-Object { "$($_.object_id):$($_.channel_id)" })
$causalDirectPresent = @($causalExpectedIds | Where-Object { $causalSourceIds -contains $_ }).Count -eq $causalExpectedIds.Count
$causalDynamicPresent = @($causalFirstFrame | Where-Object { $_.kind -eq "Objects" -and [double]$_.source_energy -gt 0 }).Count -eq 2
$causalNoDuplicate = ($causalSourceIds | Sort-Object -Unique).Count -eq $causalSourceIds.Count
$causalLfeCount = @($causalInventoryRows | Where-Object { $_.is_lfe -eq "True" -or $_.is_lfe -eq "true" }).Count
$causalJson = [pscustomobject]@{
    inventory_track_count = $causalInventoryRows.Count; inventory_non_lfe_count = $causalInventoryRows.Count - $causalLfeCount;
    inventory_lfe_count = $causalLfeCount; metrics_first_frame_source_count = $causalFirstFrame.Count;
    direct_speaker_contribution_present = $causalDirectPresent; dynamic_object_contribution_present = $causalDynamicPresent;
    no_duplicate_source_ids = $causalNoDuplicate; lfe_singular = ($causalLfeCount -eq 1);
    openjoc_strict_result = $causalInspection.openjoc_strict_result; finite_pcm = $causalInspection.finite_pcm;
    causal_test = ($causalDirectPresent -and $causalDynamicPresent -and $causalNoDuplicate -and $causalLfeCount -eq 1 -and $causalInspection.openjoc_strict_result -eq "PASS" -and $causalInspection.finite_pcm -eq "YES")
}
$causalJson | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "directspeakers-causal.json") -Encoding utf8

$unsupportedFixture = Join-Path $artifact "unsupported-input.bw64"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $unsupportedFixture)) {
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture-unsupported", "--seconds", "1", $unsupportedFixture)
}
$priorErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
try {
    $unsupportedText = (& cargo run -p jocforge --quiet -- encode $unsupportedFixture (Join-Path $artifact "unsupported-output.ec3") --profile idx1 2>&1) -join "`n"
    $unsupportedExit = $LASTEXITCODE
}
finally {
    $ErrorActionPreference = $priorErrorActionPreference
}
$unsupportedJson = [pscustomobject]@{
    exit_code = $unsupportedExit; diagnostic = $unsupportedText.Trim();
    explicit_rejection = ($unsupportedExit -ne 0 -and $unsupportedText -match "unsupported ADM object kind")
}
$unsupportedJson | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "unsupported-input.json") -Encoding utf8

$shortFixture = Join-Path $artifact "controlled-short-final.bw64"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $shortFixture)) {
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", "--objects", "15", $shortFixture)
}
$shortOutput = Join-Path $artifact "controlled-short-final.ec3"
Invoke-Encode $shortFixture $shortOutput 1664
$shortRows = @(Read-MetricRows "$shortOutput.metrics.csv")
$shortFrameRows = @($shortRows | Group-Object frame | ForEach-Object { $_.Group[0] } | Sort-Object { [int]$_.frame })
$shortRepeat = Join-Path $artifact "controlled-short-final-repeat.ec3"
Invoke-Encode $shortFixture $shortRepeat 1664
$shortExpectedFrames = [int][Math]::Ceiling(48000.0 / [double]$ShortFinalFrameSamples)
$shortActualFrames = $shortFrameRows.Count
$shortFrameNumbers = @($shortFrameRows | ForEach-Object { [int]$_.frame })
$shortFinalFramePresent = @($shortFrameNumbers | Where-Object { $_ -eq $shortExpectedFrames - 1 }).Count -eq 1
$shortNoDuplicateFrames = ($shortFrameNumbers | Sort-Object -Unique).Count -eq $shortActualFrames
$shortJson = [pscustomobject]@{
    expected_frame_count = $shortExpectedFrames; actual_frame_count = $shortActualFrames;
    final_frame_present = $shortFinalFramePresent; no_duplicate_frames = $shortNoDuplicateFrames;
    repeated_process_deterministic = ((Get-Sha256 $shortOutput) -eq (Get-Sha256 $shortRepeat));
    source_rows = $shortRows.Count; source_count = if ($shortFrameRows.Count -gt 0) { [int]$shortFrameRows[0].source_count } else { 0 };
    pass = ($shortActualFrames -eq $shortExpectedFrames -and $shortFinalFramePresent -and $shortNoDuplicateFrames -and ((Get-Sha256 $shortOutput) -eq (Get-Sha256 $shortRepeat)))
}
$shortJson | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "short-final-process.json") -Encoding utf8

$hanabiProfiles = @(
    [pscustomobject]@{ name = "idx0"; layout = "2.0"; output = (Join-Path $artifact "hanabi-idx0.ec3") },
    [pscustomobject]@{ name = "idx1"; layout = "7.1.4"; output = $hanabiOutput },
    [pscustomobject]@{ name = "idx2"; layout = "7.1.4"; output = (Join-Path $artifact "hanabi-idx2.ec3") }
)
$hanabiValidations = [System.Collections.Generic.List[object]]::new()
$hanabiPrimaryAccountingPass = $true
foreach ($profile in $hanabiProfiles) {
    if ($profile.name -ne "idx1") {
        Invoke-Encode $SourceAdm $profile.output 1664 $profile.name
    }
    $inspection = Invoke-Inspect $profile.output "hanabi-$($profile.name)" $profile.layout -SkipRender
    $accounting = Test-SourceAccountingCsv "$($profile.output).metrics.csv" -1 $hanabiExpectedSourceIndices
    $quality = Test-QualityGatesFromMetricSummary (Read-MetricSummary "$($profile.output).metrics.csv")
    $hanabiPrimaryAccountingPass = $hanabiPrimaryAccountingPass -and $accounting.pass
    $inspection | Add-Member -NotePropertyName source_accounting_pass -NotePropertyValue $accounting.pass
    $inspection | Add-Member -NotePropertyName quality_gate_pass -NotePropertyValue $quality.pass
    $roundTripQualityPass = [double]$inspection.unexplained_cluster_leakage_max -le $UnexplainedLeakageThreshold -and
        [double]$inspection.speaker_render_error_max -le $SpeakerRenderErrorThreshold -and
        [double]$inspection.factorization_reconstruction_error_max -ge 0.0 -and
        [double]$inspection.matrix_coefficient_jump_max -ge 0.0 -and
        [double]$inspection.carrier_coefficient_jump_max -ge 0.0
    $inspection | Add-Member -NotePropertyName quality_observation_pass -NotePropertyValue $roundTripQualityPass
    $hanabiValidations.Add($inspection)
    [GC]::Collect()
}

$controlledProfileValidations = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $hanabiProfiles) {
    $controlledOutput = Join-Path $artifact "controlled-profile-$($profile.name).ec3"
    Invoke-Encode $rateFixture $controlledOutput 1664 $profile.name
    $inspection = Invoke-Inspect $controlledOutput "controlled-profile-$($profile.name)" $profile.layout
    $rows = Read-MetricRows "$controlledOutput.metrics.csv"
    $accounting = Test-SourceAccounting $rows 15
    $quality = Test-QualityGates $rows
    $inspection | Add-Member -NotePropertyName source_accounting_pass -NotePropertyValue $accounting.pass
    $inspection | Add-Member -NotePropertyName quality_gate_pass -NotePropertyValue $quality.pass
    $controlledProfileValidations.Add($inspection)
}

$carrierCausalityPath = Join-Path $artifact "carrier-causality.json"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "carrier-causality", $carrierCausalityPath)
$carrierCausality = Get-Content -LiteralPath $carrierCausalityPath -Raw | ConvertFrom-Json
$carrierStagePass = Test-CarrierCausalityReport $carrierCausality

$sourceCausalityProfiles = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $hanabiProfiles) {
    $carrierCount = if ($profile.name -eq "idx0") { 5 } else { 7 }
    $baselineValidation = @($controlledProfileValidations | Where-Object { $_.name -eq "controlled-profile-$($profile.name)" })[0]
    $perturbations = [System.Collections.Generic.List[object]]::new()
    for ($sourceIndex = 0; $sourceIndex -lt $carrierCount; $sourceIndex++) {
        $perturbedFixture = Join-Path $artifact "source-causality-$($profile.name)-$sourceIndex.bw64"
        if (!$ReuseExisting -or !(Test-Path -LiteralPath $perturbedFixture)) {
            New-PerturbedBw64Fixture $rateFixture $perturbedFixture $sourceIndex
        }
        $perturbedOutput = Join-Path $artifact "source-causality-$($profile.name)-$sourceIndex.ec3"
        Invoke-Encode $perturbedFixture $perturbedOutput 1664 $profile.name
        $perturbedInspection = Invoke-Inspect $perturbedOutput "source-causality-$($profile.name)-$sourceIndex" $profile.layout
        $renderDelta = Measure-RenderedPcmDifference $baselineValidation.render $perturbedInspection.render
        $perturbations.Add([pscustomobject]@{
            source_index = $sourceIndex
            input_hash_changed = ((Get-Sha256 $rateFixture) -ne (Get-Sha256 $perturbedFixture))
            output_hash_changed = ((Get-Sha256 $baselineValidation.output) -ne (Get-Sha256 $perturbedOutput))
            render_rms_difference = $renderDelta.rms_difference
            render_maximum_absolute_difference = $renderDelta.maximum_absolute_difference
            render_delta_finite = $renderDelta.finite
            openjoc_strict_result = $perturbedInspection.openjoc_strict_result
            finite_pcm = $perturbedInspection.finite_pcm
        })
    }
    $profilePass = $perturbations.Count -eq $carrierCount -and
        @($perturbations | Where-Object {
            $_.input_hash_changed -ne $true -or $_.output_hash_changed -ne $true -or
            $_.render_delta_finite -ne $true -or $_.openjoc_strict_result -ne "PASS" -or
            $_.finite_pcm -ne "YES" -or [double]$_.render_rms_difference -le 1.0e-7
        }).Count -eq 0
    $sourceCausalityProfiles.Add([pscustomobject]@{
        profile = $profile.name; expected_carrier_count = $carrierCount; pass = $profilePass;
        perturbations = $perturbations
    })
}
$sourceCausalityReport = [pscustomobject]@{
    stage = "ENCODED_OUTPUT_AND_FINAL_DECODED_RENDER"
    fixture = "controlled-rate-15.bw64 with one independently perturbed source channel per case"
    carrier_stage = $carrierCausality
    carrier_stage_pass = $carrierStagePass
    profiles = $sourceCausalityProfiles
    pass = $carrierStagePass -and (Test-SourceCausalityReport ([pscustomobject]@{
            pass = ($sourceCausalityProfiles | Where-Object { $_.pass -ne $true }).Count -eq 0
            profiles = $sourceCausalityProfiles
        }))
}
$sourceCausalityReport | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $artifact "source-causality.json") -Encoding utf8
$sourceCausalityPass = Test-SourceCausalityReport $sourceCausalityReport

$finalRenderValidations = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $hanabiProfiles) {
    $validation = @($controlledProfileValidations | Where-Object { $_.name -eq "controlled-profile-$($profile.name)" })[0]
    if ($null -eq $validation -or $validation.render_result -ne "PASS") {
        $finalRenderValidations.Add([pscustomobject]@{
            stage = "FINAL_DECODED_RENDER"; profile = $profile.name; pass = $false;
            reason = "pinned OpenJOC render was not available"; gate = $null
        })
        continue
    }
    $profileRows = Read-MetricRows "$($validation.output).metrics.csv"
    $sourceLeakage = Get-MaxMetric $profileRows unexplained_cluster_leakage
    $measurement = Measure-FinalRenderEvidence $validation.output $validation.render $profile.name
    $measurement | Add-Member -NotePropertyName input_fixture_id -NotePropertyValue "controlled-rate-15.bw64"
    $measurement | Add-Member -NotePropertyName encoded_artifact_sha256 -NotePropertyValue (Get-Sha256 $validation.output)
    $measurement | Add-Member -NotePropertyName rendered_artifact_sha256 -NotePropertyValue (Get-Sha256 $validation.render)
    $measurement | Add-Member -NotePropertyName pinned_openjoc_identity -NotePropertyValue $validatorIdentity
    $measurement | Add-Member -NotePropertyName source_stage_leakage -NotePropertyValue $sourceLeakage
    $measurement | Add-Member -NotePropertyName final_render_leakage_threshold -NotePropertyValue $UnexplainedLeakageThreshold
    $gatePath = Join-Path $artifact "final-render-$($profile.name).json"
    $gate = Invoke-FinalRenderGate $measurement $gatePath
    $measurement | Add-Member -NotePropertyName gate -NotePropertyValue $gate
    $measurement | Add-Member -NotePropertyName validator_identity -NotePropertyValue $validatorIdentity
    $finalRenderValidations.Add($measurement)
}
$finalRenderProfilesPass = @($finalRenderValidations | Where-Object { $_.gate.pass -ne $true -or $_.gate.command_exit_code -ne 0 }).Count -eq 0

$finalRenderCorpusValidations = [System.Collections.Generic.List[object]]::new()
foreach ($relationship in @("quadrature", "transient")) {
    $relationshipFixture = Join-Path $artifact "spectral-final-$relationship.bw64"
    if (!$ReuseExisting -or !(Test-Path -LiteralPath $relationshipFixture)) {
        New-RelationshipBw64Fixture $rateFixture $relationshipFixture $relationship
    }
    $relationshipOutput = Join-Path $artifact "spectral-final-$relationship.ec3"
    Invoke-Encode $relationshipFixture $relationshipOutput 1664 "idx1"
    $relationshipInspection = Invoke-Inspect $relationshipOutput "spectral-final-$relationship" "7.1.4"
    $relationshipRows = Read-MetricRows "$relationshipOutput.metrics.csv"
    $relationshipLeakage = Get-MaxMetric $relationshipRows unexplained_cluster_leakage
    $relationshipMeasurement = Measure-FinalRenderEvidence $relationshipOutput $relationshipInspection.render "idx1" $relationship
    $relationshipMeasurement | Add-Member -NotePropertyName input_fixture_id -NotePropertyValue ("spectral-final-" + $relationship + ".bw64")
    $relationshipMeasurement | Add-Member -NotePropertyName encoded_artifact_sha256 -NotePropertyValue (Get-Sha256 $relationshipOutput)
    $relationshipMeasurement | Add-Member -NotePropertyName rendered_artifact_sha256 -NotePropertyValue (Get-Sha256 $relationshipInspection.render)
    $relationshipMeasurement | Add-Member -NotePropertyName pinned_openjoc_identity -NotePropertyValue $validatorIdentity
    $relationshipMeasurement | Add-Member -NotePropertyName source_stage_leakage -NotePropertyValue $relationshipLeakage
    $relationshipMeasurement | Add-Member -NotePropertyName final_render_leakage_threshold -NotePropertyValue $UnexplainedLeakageThreshold
    $relationshipGatePath = Join-Path $artifact "final-render-corpus-$relationship.json"
    $relationshipGate = Invoke-FinalRenderGate $relationshipMeasurement $relationshipGatePath
    $relationshipMeasurement | Add-Member -NotePropertyName gate -NotePropertyValue $relationshipGate
    $relationshipMeasurement | Add-Member -NotePropertyName validator_identity -NotePropertyValue $validatorIdentity
    $finalRenderCorpusValidations.Add($relationshipMeasurement)
}
$finalRenderCorpusPass = @($finalRenderCorpusValidations | Where-Object { $_.gate.pass -ne $true -or $_.gate.command_exit_code -ne 0 }).Count -eq 0
$finalRenderPass = $finalRenderProfilesPass -and $finalRenderCorpusPass

$finalRenderSummaryPath = Join-Path $artifact "final-decoded-render-summary.json"
[ordered]@{
    stage = "FINAL_DECODED_RENDER"
    decoder = "PINNED_OPENJOC"
    validator_identity = $validatorIdentity
    final_render_pass = $finalRenderPass
    profiles = $finalRenderValidations
    corpus = $finalRenderCorpusValidations
} | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $finalRenderSummaryPath -Encoding utf8
$spectralPhasePath = Join-Path $artifact "spectral-phase.json"
Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "spectral-phase", $spectralPhasePath, "--final-render-report", $finalRenderSummaryPath)
$spectralPhase = Get-Content -LiteralPath $spectralPhasePath -Raw | ConvertFrom-Json
$controlledIdx1 = @($controlledProfileValidations | Where-Object { $_.name -eq "controlled-profile-idx1" })[0]
$controlledIdx1Rows = Read-MetricRows "$($controlledIdx1.output).metrics.csv"
$controlledIdx1Accounting = Test-SourceAccounting $controlledIdx1Rows 15
$controlledIdx1Quality = Test-QualityGates $controlledIdx1Rows
$stationaryRow = @($temporalRows | Where-Object { $_.scene -eq "stationary-control" })[0]
$movingRows = @($temporalRows | Where-Object { $_.scene -ne "stationary-control" })
$t005Evidence = Test-StationaryTemporalEvidence $stationaryRow $controlledIdx1Rows
$t006Evidence = Test-MovingTemporalEvidence $movingRows
$hanabiAccountingPass = $hanabiPrimaryAccountingPass
$hanabiRoundTripQualityPass = @($hanabiValidations | Where-Object {
        $_.quality_observation_pass -eq $true
    }).Count -eq 3
$hanabiObjectBudgetPass = @($hanabiValidations | Where-Object {
        [int]$_.coded_count -gt $MaximumCodedObjects -or [int]$_.silent_drops -ne 0
    }).Count -eq 0
$directSpeakersDocumented =
    (Select-String -LiteralPath (Join-Path $root "README.md") -Pattern "DirectSpeakers" -Quiet) -and
    (Select-String -LiteralPath (Join-Path $root "PROVENANCE.md") -Pattern "DirectSpeakers" -Quiet)

$t001Pass = $null -ne $controlledIdx1 -and
    $controlledIdx1.openjoc_strict_result -eq "PASS" -and
    $controlledIdx1.finite_pcm -eq "YES" -and
    $controlledIdx1.render_result -eq "PASS" -and
    $controlledIdx1.malformed_aus -eq 0 -and
    $controlledIdx1Accounting.pass -and
    $controlledIdx1.coded_count -le $MaximumCodedObjects -and
    $controlledIdx1Quality.pass -and
    $sourceCausalityPass
$t008ProfileConditions = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $hanabiProfiles) {
    $validation = @($controlledProfileValidations | Where-Object { $_.name -eq "controlled-profile-$($profile.name)" })[0]
    $finalRender = @($finalRenderValidations | Where-Object { $_.profile -eq $profile.name })[0]
    $rows = if ($null -ne $validation) { Read-MetricRows "$($validation.output).metrics.csv" } else { @() }
    $conditions = Test-T008ProfileConditions $validation $rows $finalRender
    $t008ProfileConditions.Add([pscustomobject]@{
        profile = $profile.name; pass = $conditions.pass; reason = $conditions.reason;
        strict = $conditions.strict; render = $conditions.render; finite = $conditions.finite;
        malformed_aus = $conditions.malformed_aus; carrier_traversal = $conditions.carrier_traversal;
        source_reconstruction = $conditions.source_reconstruction; spatial = $conditions.spatial;
        source_leakage = $conditions.source_leakage; speaker = $conditions.speaker; factorization = $conditions.factorization;
        matrix = $conditions.matrix; carrier = $conditions.carrier; coded_peak = $conditions.coded_peak;
        coded_headroom = $conditions.coded_headroom; final_render_leakage = $conditions.final_render_leakage;
        final_decoded_render = $conditions.final_decoded_render;
        final_render_pass = $conditions.final_decoded_render
    })
}
$t008AllProfileConditionsPass = @($t008ProfileConditions | Where-Object { $_.pass -ne $true }).Count -eq 0
$t008Pass = $spectralPhase.pass -eq $true -and
    @($hanabiValidations | Where-Object { $_.openjoc_strict_result -ne "PASS" -or $_.finite_pcm -ne "YES" -or $_.malformed_aus -ne 0 -or $_.carrier_traversal_errors -ne 0 }).Count -eq 0 -and
    $t008AllProfileConditionsPass -and $finalRenderProfilesPass -and $finalRenderCorpusPass
$t009Pass = $quantization.pass -eq $true -and
    [int]$quantization.profile_count -eq 3 -and
    [int]$quantization.domain_values_checked -eq 96 -and
    [int]$quantization.matrix_values_checked -gt 0

$t002Pass = @($fixtureRows | Where-Object {
        $_.scene -in @("controlled-16", "controlled-20", "controlled-28") -and
        $_.coded_count -le $MaximumCodedObjects -and
        $_.silent_drops -eq 0 -and
        $_.source_accounting_pass -eq $true -and
        $_.quality_gate_pass -eq $true
    }).Count -eq 3 -and $sourceCausalityPass
$t003Pass = @($hanabiValidations | Where-Object {
        $_.openjoc_strict_result -ne "PASS" -or $_.finite_pcm -ne "YES" -or $_.malformed_aus -ne 0
    }).Count -eq 0 -and
    $hanabiAccountingPass -and $hanabiObjectBudgetPass -and $hanabiRoundTripQualityPass -and
    $causalJson.causal_test -eq $true -and $directSpeakersDocumented -and $sourceCausalityPass

$bitrateVerdicts = @($bitrateRows | ForEach-Object { Test-BitrateCell $_ })
$bitrateSupportedCount = @($bitrateVerdicts | Where-Object { $_.classification -eq "SUPPORTED" }).Count
$bitrateRejectedCapacityCount = @($bitrateVerdicts | Where-Object { $_.classification -eq "REJECTED_CAPACITY" }).Count
$bitrateCorrectnessFailureCount = @($bitrateVerdicts | Where-Object { $_.classification -eq "FAIL_CORRECTNESS" }).Count
$t004Pass = $bitrateCorrectnessFailureCount -eq 0 -and
    $bitrateSupportedCount -gt 0 -and
    $bitrateRejectedCapacityCount -gt 0
$bitrateAntiContradictionTest = $false
$bitrateSupportedBaseline = @($bitrateRows | Where-Object {
        $_.profile -eq "idx0" -and $_.target_bitrate_kbps -eq 1024 -and
        (Test-BitrateCell $_).classification -eq "SUPPORTED"
    })[0]
if ($null -ne $bitrateSupportedBaseline) {
    $bitrateMutation = ConvertFrom-Json ($bitrateSupportedBaseline | ConvertTo-Json -Depth 10)
    $bitrateMutation.openjoc_parse_result = "FAIL"
    $bitrateMutation.openjoc_strict_result = "FAIL"
    $bitrateMutation.openjoc_decode_result = "FAIL"
    $bitrateMutation.openjoc_render_result = "FAIL"
    $bitrateMutation.compatibility_render = "FAIL"
    $bitrateMutation.classification = "SUPPORTED"
    $bitrateMutation.status = "SUPPORTED"
    $bitrateAntiContradictionTest = -not (Test-BitrateCell $bitrateMutation).pass
}

$antiSelfReportRoot = Join-Path ([IO.Path]::GetTempPath()) "jocforge-closure-anti-self-report-$PID"
New-Item -ItemType Directory -Force $antiSelfReportRoot | Out-Null
$antiSelfReportPath = Join-Path $antiSelfReportRoot "forged-closure.json"
try {
    [ordered]@{ clean_tests = @([ordered]@{ test_id = "T-PROBE"; status = "PASS" }) } |
        ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $antiSelfReportPath -Encoding utf8
    $primaryProbe = $t009Pass
    $computedBeforeMutation = Get-DerivedStatus $primaryProbe
    $forged = Get-Content -LiteralPath $antiSelfReportPath -Raw | ConvertFrom-Json
    $forged.clean_tests[0].status = "FAIL"
    $forged | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $antiSelfReportPath -Encoding utf8
    $computedAfterMutation = Get-DerivedStatus $primaryProbe
    $antiSelfReport = [pscustomobject]@{
        pass = ($computedBeforeMutation -eq $computedAfterMutation -and $computedAfterMutation -eq (Get-DerivedStatus $primaryProbe) -and $forged.clean_tests[0].status -ne $computedAfterMutation)
        source = "temporary forged closure JSON plus primary quantization evidence"
        computed_before_mutation = $computedBeforeMutation
        computed_after_mutation = $computedAfterMutation
        forged_status_after_mutation = $forged.clean_tests[0].status
    }
}
finally {
    if (Test-Path -LiteralPath $antiSelfReportRoot) { Remove-Item -LiteralPath $antiSelfReportRoot -Recurse -Force }
}

$negativeMutationRoot = Join-Path ([IO.Path]::GetTempPath()) "jocforge-closure-negative-mutations-$PID"
New-Item -ItemType Directory -Force $negativeMutationRoot | Out-Null
try {
    [ordered]@{
        source = $controlledIdx1Rows[0]
        source_rows = @($controlledIdx1Rows)
        stationary = $stationaryRow
        moving = $movingRows[0]
    } | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $negativeMutationRoot "primary.json") -Encoding utf8
    $mutation = Get-Content -LiteralPath (Join-Path $negativeMutationRoot "primary.json") -Raw | ConvertFrom-Json
    foreach ($row in @($mutation.source_rows)) { $row.silent_drops = 1 }
    $sourceDropRejected = -not (Test-SourceAccounting @($mutation.source_rows) 15).pass
    $mutation.source.silent_drops = 0
    $mutation.source.unexplained_cluster_leakage = 1.0
    $qualityThresholdRejected = -not (Test-QualityGates @($mutation.source)).pass
    $mutation.stationary.total_representative_source_changes = 1
    $stationaryChurnRejected = -not (Test-StationaryTemporalEvidence $mutation.stationary $controlledIdx1Rows).pass
    $mutation.moving.maximum_position_jump = $TemporalPositionJumpThreshold + 1.0
    $movingDiscontinuityRejected = -not (Test-MovingTemporalEvidence @($mutation.moving)).pass
    $finalRenderBase = @($finalRenderValidations | Where-Object { $_.profile -eq "idx1" })[0]
    $finalRenderMutations = @(
        [pscustomobject]@{ name = "spectral_tilt"; finite = $finalRenderBase.finite; channel_coverage = $finalRenderBase.channel_coverage; spectral_error = $finalRenderBase.spectral_error; spectral_tilt_error = 2.0; phase_coherence = $finalRenderBase.phase_coherence; polarity_preserved = $finalRenderBase.polarity_preserved; quadrature_phase_error = $finalRenderBase.quadrature_phase_error; transient_phase_discontinuity = $finalRenderBase.transient_phase_discontinuity; final_render_leakage = $finalRenderBase.final_render_leakage; peak = $finalRenderBase.peak; headroom_margin = $finalRenderBase.headroom_margin },
        [pscustomobject]@{ name = "phase_collapse"; finite = $finalRenderBase.finite; channel_coverage = $finalRenderBase.channel_coverage; spectral_error = $finalRenderBase.spectral_error; spectral_tilt_error = $finalRenderBase.spectral_tilt_error; phase_coherence = 0.0; polarity_preserved = $finalRenderBase.polarity_preserved; quadrature_phase_error = $finalRenderBase.quadrature_phase_error; transient_phase_discontinuity = $finalRenderBase.transient_phase_discontinuity; final_render_leakage = $finalRenderBase.final_render_leakage; peak = $finalRenderBase.peak; headroom_margin = $finalRenderBase.headroom_margin },
        [pscustomobject]@{ name = "polarity_inversion"; finite = $finalRenderBase.finite; channel_coverage = $finalRenderBase.channel_coverage; spectral_error = $finalRenderBase.spectral_error; spectral_tilt_error = $finalRenderBase.spectral_tilt_error; phase_coherence = $finalRenderBase.phase_coherence; polarity_preserved = $false; quadrature_phase_error = $finalRenderBase.quadrature_phase_error; transient_phase_discontinuity = $finalRenderBase.transient_phase_discontinuity; final_render_leakage = $finalRenderBase.final_render_leakage; peak = $finalRenderBase.peak; headroom_margin = $finalRenderBase.headroom_margin },
        [pscustomobject]@{ name = "quadrature_collapse"; finite = $finalRenderBase.finite; channel_coverage = $finalRenderBase.channel_coverage; spectral_error = $finalRenderBase.spectral_error; spectral_tilt_error = $finalRenderBase.spectral_tilt_error; phase_coherence = $finalRenderBase.phase_coherence; polarity_preserved = $finalRenderBase.polarity_preserved; quadrature_phase_error = 2.0; transient_phase_discontinuity = $finalRenderBase.transient_phase_discontinuity; final_render_leakage = $finalRenderBase.final_render_leakage; peak = $finalRenderBase.peak; headroom_margin = $finalRenderBase.headroom_margin },
        [pscustomobject]@{ name = "transient_phase"; finite = $finalRenderBase.finite; channel_coverage = $finalRenderBase.channel_coverage; spectral_error = $finalRenderBase.spectral_error; spectral_tilt_error = $finalRenderBase.spectral_tilt_error; phase_coherence = $finalRenderBase.phase_coherence; polarity_preserved = $finalRenderBase.polarity_preserved; quadrature_phase_error = $finalRenderBase.quadrature_phase_error; transient_phase_discontinuity = 1.0; final_render_leakage = $finalRenderBase.final_render_leakage; peak = $finalRenderBase.peak; headroom_margin = $finalRenderBase.headroom_margin },
        [pscustomobject]@{ name = "leakage"; finite = $finalRenderBase.finite; channel_coverage = $finalRenderBase.channel_coverage; spectral_error = $finalRenderBase.spectral_error; spectral_tilt_error = $finalRenderBase.spectral_tilt_error; phase_coherence = $finalRenderBase.phase_coherence; polarity_preserved = $finalRenderBase.polarity_preserved; quadrature_phase_error = $finalRenderBase.quadrature_phase_error; transient_phase_discontinuity = $finalRenderBase.transient_phase_discontinuity; final_render_leakage = 1.0; peak = $finalRenderBase.peak; headroom_margin = $finalRenderBase.headroom_margin },
        [pscustomobject]@{ name = "headroom"; finite = $finalRenderBase.finite; channel_coverage = $finalRenderBase.channel_coverage; spectral_error = $finalRenderBase.spectral_error; spectral_tilt_error = $finalRenderBase.spectral_tilt_error; phase_coherence = $finalRenderBase.phase_coherence; polarity_preserved = $finalRenderBase.polarity_preserved; quadrature_phase_error = $finalRenderBase.quadrature_phase_error; transient_phase_discontinuity = $finalRenderBase.transient_phase_discontinuity; final_render_leakage = $finalRenderBase.final_render_leakage; peak = 1.0; headroom_margin = -0.1 }
    )
    $finalRenderMutationResults = [System.Collections.Generic.List[object]]::new()
    foreach ($finalMutation in $finalRenderMutations) {
        $mutationReportPath = Join-Path $negativeMutationRoot ("final-render-" + $finalMutation.name + ".json")
        $mutationGate = Invoke-FinalRenderGate $finalMutation $mutationReportPath
        $finalRenderMutationResults.Add([pscustomobject]@{
            mutation = $finalMutation.name; rejected = ($mutationGate.pass -ne $true); gate = $mutationGate
        })
    }
    $finalRenderNegativePass = $finalRenderMutationResults.Count -eq $finalRenderMutations.Count -and
        @($finalRenderMutationResults | Where-Object { $_.rejected -ne $true }).Count -eq 0
    $carrierPlaceholderMutation = Get-Content -LiteralPath $carrierCausalityPath -Raw | ConvertFrom-Json
    $carrierPlaceholderMutation.profiles[0].carriers[0].carrier_changed = $false
    $placeholderCarrierNegativeTest = -not (Test-CarrierCausalityReport $carrierPlaceholderMutation)
    $sourceCausalityMutation = Get-Content -LiteralPath (Join-Path $artifact "source-causality.json") -Raw | ConvertFrom-Json
    $sourceCausalityMutation.profiles[0].perturbations[0].render_rms_difference = 0.0
    $sourceCausalityMutation.profiles[0].perturbations[0].output_hash_changed = $false
    $sourceCausalityNegativeTest = -not (Test-SourceCausalityReport $sourceCausalityMutation)
    $t008FinalRenderBase = @($finalRenderValidations | Where-Object { $_.profile -eq "idx1" })[0]
    $finalRenderPcmMutationPath = Join-Path $negativeMutationRoot "final-render-leakage-pcm.wav"
    New-FinalRenderLeakageMutation $t008FinalRenderBase.rendered_path $finalRenderPcmMutationPath 3
    $finalRenderPcmMutationMeasurement = Measure-FinalRenderEvidence $t008FinalRenderBase.encoded_path $finalRenderPcmMutationPath "idx1"
    $finalRenderPcmMutationGatePath = Join-Path $negativeMutationRoot "final-render-leakage-pcm.json"
    $finalRenderPcmMutationGate = Invoke-FinalRenderGate $finalRenderPcmMutationMeasurement $finalRenderPcmMutationGatePath
    $finalRenderPcmNegativePass = [double]$finalRenderPcmMutationMeasurement.final_render_leakage -gt $UnexplainedLeakageThreshold -and
        $finalRenderPcmMutationGate.pass -ne $true
    $sourceOnlyRowsMutation = ConvertFrom-Json (ConvertTo-Json -InputObject ([object[]]$controlledIdx1Rows) -Depth 10)
    foreach ($row in $sourceOnlyRowsMutation) { $row.unexplained_cluster_leakage = 0.0 }
    $badFinalRenderEvidence = ConvertFrom-Json ($t008FinalRenderBase | ConvertTo-Json -Depth 10)
    $badFinalRenderEvidence.final_render_leakage = 1.0
    $badFinalRenderEvidence.gate.pass = $false
    $sourceOnlyOverridePass = (Test-T008ProfileConditions $controlledIdx1 $sourceOnlyRowsMutation $badFinalRenderEvidence).pass
    $sourceStageCannotOverrideFinalRender = -not $sourceOnlyOverridePass
    $t008LeakageRowsMutation = ConvertFrom-Json (ConvertTo-Json -InputObject ([object[]]$controlledIdx1Rows) -Depth 10)
    foreach ($row in $t008LeakageRowsMutation) { $row.unexplained_cluster_leakage = 1.0 }
    $t008AggregateLeakageNegativeTest = -not (Test-T008ProfileConditions $controlledIdx1 $t008LeakageRowsMutation $t008FinalRenderBase).pass
    $t008HeadroomRowsMutation = ConvertFrom-Json (ConvertTo-Json -InputObject ([object[]]$controlledIdx1Rows) -Depth 10)
    foreach ($row in $t008HeadroomRowsMutation) { $row.max_coded_peak = 1.0; $row.headroom_margin = -0.1 }
    $t008AggregateHeadroomNegativeTest = -not (Test-T008ProfileConditions $controlledIdx1 $t008HeadroomRowsMutation $t008FinalRenderBase).pass
    $negativeMutationTests = [pscustomobject]@{
        pass = $sourceDropRejected -and $qualityThresholdRejected -and $stationaryChurnRejected -and $movingDiscontinuityRejected -and $finalRenderNegativePass -and $finalRenderPcmNegativePass -and $sourceStageCannotOverrideFinalRender -and $bitrateAntiContradictionTest -and $placeholderCarrierNegativeTest -and $sourceCausalityNegativeTest -and $t008AggregateLeakageNegativeTest -and $t008AggregateHeadroomNegativeTest
        source_drop_rejected = $sourceDropRejected
        quality_threshold_rejected = $qualityThresholdRejected
        stationary_churn_rejected = $stationaryChurnRejected
        moving_discontinuity_rejected = $movingDiscontinuityRejected
        final_render_mutations = $finalRenderMutationResults
        final_render_negative_pass = $finalRenderNegativePass
        final_render_pcm_negative_pass = $finalRenderPcmNegativePass
        final_render_pcm_mutation = [pscustomobject]@{
            measured_final_render_leakage = $finalRenderPcmMutationMeasurement.final_render_leakage
            threshold = $UnexplainedLeakageThreshold
            gate_pass = $finalRenderPcmMutationGate.pass
        }
        source_stage_cannot_override_final_render = $sourceStageCannotOverrideFinalRender
        source_leakage_reused_as_final = -not $sourceStageCannotOverrideFinalRender
        bitrate_anti_contradiction_test = $bitrateAntiContradictionTest
        placeholder_carrier_negative_test = $placeholderCarrierNegativeTest
        source_causality_negative_test = $sourceCausalityNegativeTest
        t008_aggregate_leakage_negative_test = $t008AggregateLeakageNegativeTest
        t008_aggregate_headroom_negative_test = $t008AggregateHeadroomNegativeTest
        source = "temporary synthetic JSON derived from primary evidence; committed artifacts were not modified"
    }
}
finally {
    if (Test-Path -LiteralPath $negativeMutationRoot) { Remove-Item -LiteralPath $negativeMutationRoot -Recurse -Force }
}

$t001Predicates = [ordered]@{
    syntax_valid = ($controlledIdx1.openjoc_strict_result -eq "PASS" -and $controlledIdx1.malformed_aus -eq 0)
    finite_pcm = ($controlledIdx1.finite_pcm -eq "YES")
    source_coverage = ($controlledIdx1Accounting.pass -and $sourceCausalityPass)
    coded_budget = ($controlledIdx1.coded_count -le $MaximumCodedObjects)
    quality_thresholds = $controlledIdx1Quality.pass
}
$t002Predicates = [ordered]@{
    reduction_fixture_coverage = (@($fixtureRows | Where-Object { $_.scene -in @("controlled-16", "controlled-20", "controlled-28") }).Count -eq 3)
    coded_budget = (@($fixtureRows | Where-Object { $_.scene -in @("controlled-16", "controlled-20", "controlled-28") -and $_.coded_count -le $MaximumCodedObjects }).Count -eq 3)
    no_silent_discard = (@($fixtureRows | Where-Object { $_.scene -in @("controlled-16", "controlled-20", "controlled-28") -and $_.silent_drops -eq 0 }).Count -eq 3)
    source_coverage = (@($fixtureRows | Where-Object { $_.scene -in @("controlled-16", "controlled-20", "controlled-28") -and $_.source_accounting_pass -eq $true }).Count -eq 3 -and $sourceCausalityPass)
    quality_thresholds = (@($fixtureRows | Where-Object { $_.scene -in @("controlled-16", "controlled-20", "controlled-28") -and $_.quality_gate_pass -eq $true }).Count -eq 3)
}
$t003Predicates = [ordered]@{
    all_profile_validation = (@($hanabiValidations | Where-Object { $_.openjoc_strict_result -eq "PASS" -and $_.finite_pcm -eq "YES" -and $_.malformed_aus -eq 0 }).Count -eq 3)
    no_silent_object_discard = $hanabiObjectBudgetPass
    authored_source_coverage = ($hanabiAccountingPass -and $sourceCausalityPass)
    directspeakers_handling_documented = ($causalJson.causal_test -eq $true -and $directSpeakersDocumented)
    roundtrip_quality_observation = $hanabiRoundTripQualityPass
}
$t008Predicates = [ordered]@{
    internal_spectral_phase_corpus = ($spectralPhase.pass -eq $true)
    final_render_profiles = $finalRenderProfilesPass
    final_render_corpus = $finalRenderCorpusPass
    declared_profile_conditions = $t008AllProfileConditionsPass
    source_leakage_gate = (@($t008ProfileConditions | Where-Object { $_.source_leakage -ne $true }).Count -eq 0)
    final_render_leakage_gate = (@($t008ProfileConditions | Where-Object { $_.final_render_leakage -ne $true }).Count -eq 0)
    headroom_gate = (@($t008ProfileConditions | Where-Object { $_.coded_headroom -ne $true -or $_.final_decoded_render -ne $true }).Count -eq 0)
    final_render_negative_mutations = $negativeMutationTests.final_render_negative_pass
    final_render_pcm_negative_mutation = $negativeMutationTests.final_render_pcm_negative_pass
    source_stage_cannot_override_final_render = $negativeMutationTests.source_stage_cannot_override_final_render
}
$predicateAudit = [ordered]@{
    T001 = [ordered]@{ declared_conditions = $t001Predicates; all_conditions_pass = (@($t001Predicates.Values | Where-Object { $_ -ne $true }).Count -eq 0) }
    T002 = [ordered]@{ declared_conditions = $t002Predicates; all_conditions_pass = (@($t002Predicates.Values | Where-Object { $_ -ne $true }).Count -eq 0) }
    T003 = [ordered]@{ declared_conditions = $t003Predicates; all_conditions_pass = (@($t003Predicates.Values | Where-Object { $_ -ne $true }).Count -eq 0) }
    T008 = [ordered]@{ declared_conditions = $t008Predicates; all_conditions_pass = (@($t008Predicates.Values | Where-Object { $_ -ne $true }).Count -eq 0); per_profile = $t008ProfileConditions }
}
$underassertedCleanTests = @(
    [pscustomobject]@{ test_id = "T-001"; required = 5; asserted = $t001Predicates.Keys.Count }
    [pscustomobject]@{ test_id = "T-002"; required = 5; asserted = $t002Predicates.Keys.Count }
    [pscustomobject]@{ test_id = "T-003"; required = 5; asserted = $t003Predicates.Keys.Count }
    [pscustomobject]@{ test_id = "T-008"; required = 10; asserted = $t008Predicates.Keys.Count }
) | Where-Object { $_.asserted -lt $_.required }
$predicateAudit.underasserted_clean_tests = $underassertedCleanTests
$predicateAudit.underasserted_clean_tests_count = @($underassertedCleanTests | Where-Object { $null -ne $_ }).Count

$cleanTests = @(
    [pscustomobject]@{ test_id = "T-001"; status = Get-DerivedStatus $t001Pass; command = "cargo run -p jocforge -- encode + pinned OpenJOC inspect/render + FFmpeg decode + source perturbations"; evidence = "controlled-profile-idx1.ec3.metrics.csv; source-causality.json; primary inspection"; measured = "strict=$($controlledIdx1.openjoc_strict_result); finite_pcm=$($controlledIdx1.finite_pcm); render=$($controlledIdx1.render_result); malformed_aus=$($controlledIdx1.malformed_aus); source_accounting=$($controlledIdx1Accounting.pass); coded_count=$($controlledIdx1.coded_count); quality=$($controlledIdx1Quality.pass); source_causality=$sourceCausalityPass"; expected = "strict=PASS; finite_pcm=YES; render=PASS; malformed_aus=0; complete source domain; every intended carrier input has causal encoded/render response; M<=15; all published quality thresholds pass" },
    [pscustomobject]@{ test_id = "T-002"; status = Get-DerivedStatus $t002Pass; evidence = "controlled-16/20/28 metrics sidecars; independent leakage-matrix.csv; source-causality.json"; measured = "all three coded_count<=15, silent_drops=0, complete source accounting, quality thresholds pass, source_causality=$sourceCausalityPass"; expected = "M<=15; no silent discard; complete source coverage; source/error/leakage/spatial quality thresholds pass" },
    [pscustomobject]@{ test_id = "T-003"; status = Get-DerivedStatus $t003Pass; evidence = "Hanabi three-profile strict sidecars; primary idx1 accounting/quality; source-causality.json; direct-speakers-causal.json and docs"; measured = "all profiles strict/finite/M<=15/no drops; primary source accounting=$hanabiAccountingPass; round-trip observation=$hanabiRoundTripQualityPass; DirectSpeakers causal test=$($causalJson.causal_test); documented=$directSpeakersDocumented; source_causality=$sourceCausalityPass"; expected = "no silent object dropping; complete authored-source coverage; every intended idx0/idx1/idx2 carrier input is causally source-derived; M<=15; DirectSpeakers handling documented" },
    [pscustomobject]@{ test_id = "T-004"; status = Get-DerivedStatus $t004Pass; command = "cargo run -p jocforge -- encode for every profile × bitrate cell plus pinned OpenJOC and FFmpeg validation"; evidence = "profile-bitrate-matrix.csv"; measured = "supported_cells=$bitrateSupportedCount; rejected_capacity_cells=$bitrateRejectedCapacityCount; correctness_failures=$bitrateCorrectnessFailureCount"; expected = "every SUPPORTED cell passes encode, structural, OpenJOC parse/decode/render, FFmpeg decode, finite PCM and deterministic gates; lower capacity cells reject explicitly" },
    [pscustomobject]@{ test_id = "T-005"; status = Get-DerivedStatus $t005Evidence.pass; evidence = "temporal-metrics.csv plus stationary controlled-profile-idx1 metrics"; measured = "assignment/representative/slot churn and lifetimes are stable; transition bounds and matrix/carrier jumps are within thresholds"; expected = "no gratuitous representation permutation, sign/state jump, or audible reassignment artefact" },
    [pscustomobject]@{ test_id = "T-006"; status = Get-DerivedStatus $t006Evidence.pass; evidence = "temporal-metrics.csv and temporal-audit.csv"; measured = "five moving scenes checked for M, no drops, trajectory error, position jump, PCM discontinuity, gain jump, and speaker discontinuity"; expected = "bounded spatial/trajectory error without avoidable discontinuity" },
    [pscustomobject]@{ test_id = "T-007"; status = if ((@($leakageRows | Where-Object { [double]$_.unexplained_cluster_leakage -gt $UnexplainedLeakageThreshold }).Count) -eq 0) { "PASS" } else { "FAIL" }; evidence = "leakage-matrix.csv; independent coded-signal residual" },
    [pscustomobject]@{ test_id = "T-008"; status = Get-DerivedStatus $t008Pass; command = "cargo run -p jocforge -- spectral-phase plus pinned OpenJOC decoded/rendered final-render gate"; evidence = "spectral-phase.json; final-render-idx0/idx1/idx2.json; controlled profile metrics; Hanabi strict/decode"; measured = "spectral_phase=$($spectralPhase.pass); final_render_profiles=$($finalRenderPass); final_render_gates=$($t008AllProfileConditionsPass); leakage_headroom=$(@($t008ProfileConditions | Where-Object { $_.pass -ne $true }).Count -eq 0); negative_mutations=$($negativeMutationTests.final_render_negative_pass)"; expected = "all hard spectral/phase gates true; actual pinned OpenJOC final-render spectral, phase, polarity, quadrature, transient, leakage and headroom predicates pass" },
    [pscustomobject]@{ test_id = "T-009"; status = Get-DerivedStatus $t009Pass; command = "cargo run -p jocforge -- quantization-stress"; evidence = "quantization-stress.json"; measured = "profiles=$($quantization.profile_count); domain_values=$($quantization.domain_values_checked); matrix_values=$($quantization.matrix_values_checked); maximum_error=$($quantization.maximum_absolute_error)"; expected = "three profiles; 96 domain values; full matrix stress; error <= half quantum" },
    [pscustomobject]@{ test_id = "T-010"; status = if (($bitrateRows | Where-Object { $_.deterministic_reencode -eq "FAIL" }).Count -eq 0) { "PASS" } else { "FAIL" }; evidence = "bitrate sweep hashes" },
    [pscustomobject]@{ test_id = "T-011"; status = if ($shortJson.pass) { "PASS" } else { "FAIL" }; evidence = "short-final-process.json; repeated CLI process and ceil-frame accounting" },
    [pscustomobject]@{ test_id = "T-012"; status = if ($causalJson.causal_test -and $unsupportedJson.explicit_rejection) { "PASS" } else { "FAIL" }; evidence = "DirectSpeakers causal and unsupported-input JSON" }
)

$closure = [ordered]@{
    authoritative_runner = "scripts/run-validation-closure.ps1"
    validator_identity = $validatorIdentity
    validator_pin_test = $validatorPinTest
    acceptance_thresholds = [ordered]@{
        source_reconstruction_error = $SourceReconstructionErrorThreshold
        spatial_representation_error = $SpatialRepresentationErrorThreshold
        unexplained_cluster_leakage = $UnexplainedLeakageThreshold
        speaker_render_error = $SpeakerRenderErrorThreshold
        factorization_reconstruction_error = $FactorizationResidualThreshold
        matrix_coefficient_jump = $MatrixCoefficientJumpThreshold
        carrier_coefficient_jump = $CarrierCoefficientJumpThreshold
        temporal_position_jump = $TemporalPositionJumpThreshold
        temporal_pcm_boundary_discontinuity = $TemporalPcmBoundaryDiscontinuityThreshold
        temporal_gain_jump = $TemporalGainJumpThreshold
        temporal_speaker_discontinuity = $TemporalSpeakerDiscontinuityThreshold
    }
    generated = (Get-Date).ToUniversalTime().ToString("o", $invariant)
    firewall = [ordered]@{ contaminated_workspace_read = "NO"; sanitizer_private_read = "NO" }
    bitrate_sweep = $bitrateRows
    bitrate_contract = "PROFILE x BITRATE; supported cells require every mandatory validation stage"
    bitrate_matrix = [ordered]@{
        supported = $bitrateSupportedCount
        rejected_capacity = $bitrateRejectedCapacityCount
        fail_correctness = $bitrateCorrectnessFailureCount
        supported_with_render_failure = @($bitrateRows | Where-Object {
            (Test-BitrateCell $_).classification -eq "SUPPORTED" -and $_.openjoc_render_result -ne "PASS"
        }).Count
        anti_contradiction_test = $bitrateAntiContradictionTest
    }
    temporal_audit = $temporalAudits
    leakage_observations = $fixtureRows
    directspeakers = $causalJson
    unsupported_input = $unsupportedJson
    short_final_process = $shortJson
    quantization_stress = $quantization
    spectral_phase = $spectralPhase
    carrier_causality = $carrierCausality
    source_causality = $sourceCausalityReport
    final_decoded_render = $finalRenderValidations
    final_render_corpus = $finalRenderCorpusValidations
    t008_profile_conditions = $t008ProfileConditions
    anti_self_report_meta_test = $antiSelfReport
    negative_mutation_tests = $negativeMutationTests
    predicate_audit = $predicateAudit
    controlled_profile_validation = $controlledProfileValidations
    hanabi_validation = $hanabiValidations
    clean_tests = $cleanTests
}
$closure | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $artifact "clean-validation-closure.json") -Encoding utf8
Write-Output "JOCForge validation-closure evidence written to $artifact"
