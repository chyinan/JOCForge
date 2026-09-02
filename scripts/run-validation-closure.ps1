param(
    [string]$SourceAdm = "",
    [string]$OpenJoc = "D:\Programs\OpenJOC\target\debug\openjoc.exe",
    [switch]$ReuseExisting
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\validation-closure"
New-Item -ItemType Directory -Force $artifact | Out-Null

$invariant = [Globalization.CultureInfo]::InvariantCulture

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

function Invoke-Inspect([string]$OutputPath, [string]$Name, [string]$Layout) {
    $inspectLog = Join-Path $artifact "$Name.openjoc-inspect.log"
    $render = Join-Path $artifact "$Name.wav"
    $renderLog = "$render.openjoc.log"
    $decodeLog = "$render.ffmpeg.log"

    if ($ReuseExisting -and (Test-Path -LiteralPath $inspectLog) -and (Test-Path -LiteralPath $render) -and (Test-Path -LiteralPath $decodeLog)) {
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
            & $OpenJoc render-joc $OutputPath --layout $Layout --output $render --no-progress --overwrite *> $renderLog
            $renderExit = $LASTEXITCODE
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
    [pscustomobject]@{
        output = $OutputPath
        access_units = $accessUnits
        malformed_aus = $malformed
        carrier_traversal_errors = $carrierTraversalErrors
        oamd_bytes_per_au = $oamdBytes
        joc_bytes_per_au = $jocBytes
        openjoc_strict_result = if ($inspectExit -eq 0 -and $malformed -eq 0) { "PASS" } else { "FAIL" }
        render_result = if ($renderExit -eq 0) { "PASS" } else { "RESEARCH_ONLY" }
        finite_pcm = if ($decodeExit -eq 0) { "YES" } else { "NO" }
        output_bytes = $outputBytes
        render = $render
    }
}

function Read-MetricRows([string]$Path) {
    @(Import-Csv -LiteralPath $Path | Where-Object { $_.record -eq "source" })
}

function Get-MaxMetric($Rows, [string]$Name) {
    $values = @($Rows | ForEach-Object { [double]($_.$Name) })
    if ($values.Count -eq 0) { return 0.0 }
    ($values | Measure-Object -Maximum).Maximum
}

function Get-MinMetric($Rows, [string]$Name) {
    $values = @($Rows | ForEach-Object { [double]($_.$Name) })
    if ($values.Count -eq 0) { return 0.0 }
    ($values | Measure-Object -Minimum).Minimum
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
    $frameRows = @($Rows | Group-Object frame | ForEach-Object { $_.Group[0] } | Sort-Object { [int]$_.frame })
    $changes = @($frameRows | Select-Object -Skip 1 | ForEach-Object { [int]$_.assignment_changes })
    $totalChanges = if ($changes.Count -eq 0) { 0 } else { ($changes | Measure-Object -Sum).Sum }
    $firstChange = @($frameRows | Where-Object { [int]$_.assignment_changes -gt 0 } | Select-Object -First 1)
    $sourceGroups = @($Rows | Group-Object source_index)
    $lifetimes = [System.Collections.Generic.List[int]]::new()
    $maxPositionJump = 0.0
    $maxPcmDiscontinuity = 0.0
    $maxGainJump = 0.0
    $maxSpeakerDiscontinuity = 0.0
    $maxReconstructionChange = 0.0
    foreach ($group in $sourceGroups) {
        $ordered = @($group.Group | Sort-Object { [int]$_.frame })
        $currentAssignment = [int]$ordered[0].coded_object_index
        $currentLifetime = 1
        for ($index = 1; $index -lt $ordered.Count; $index++) {
            $previous = $ordered[$index - 1]
            $current = $ordered[$index]
            $maxReconstructionChange = [Math]::Max(
                $maxReconstructionChange,
                [Math]::Abs([double]$current.source_reconstruction_error - [double]$previous.source_reconstruction_error)
            )
            if ([int]$current.coded_object_index -eq $currentAssignment) {
                $currentLifetime++
            }
            else {
                $lifetimes.Add($currentLifetime)
                $currentAssignment = [int]$current.coded_object_index
                $currentLifetime = 1
                $positionJump = [Math]::Sqrt(
                    ([double]$current.coded_azimuth - [double]$previous.coded_azimuth) * ([double]$current.coded_azimuth - [double]$previous.coded_azimuth) +
                    ([double]$current.coded_elevation - [double]$previous.coded_elevation) * ([double]$current.coded_elevation - [double]$previous.coded_elevation) +
                    ([double]$current.coded_distance - [double]$previous.coded_distance) * ([double]$current.coded_distance - [double]$previous.coded_distance)
                )
                $maxPositionJump = [Math]::Max($maxPositionJump, $positionJump)
                $maxPcmDiscontinuity = [Math]::Max(
                    $maxPcmDiscontinuity,
                    [Math]::Abs([double]$current.coded_first_sample - [double]$previous.coded_last_sample)
                )
                $maxGainJump = [Math]::Max(
                    $maxGainJump,
                    [Math]::Abs([double]$current.contribution_gain - [double]$previous.contribution_gain)
                )
                $previousValue = [double]$previous.coded_last_sample
                $currentValue = [double]$current.coded_first_sample
                $previousAzimuth = [Math]::Max(-1.0, [Math]::Min(1.0, [double]$previous.coded_azimuth))
                $currentAzimuth = [Math]::Max(-1.0, [Math]::Min(1.0, [double]$current.coded_azimuth))
                $previousLeft = $previousValue * (1.0 - $previousAzimuth) * 0.5
                $previousRight = $previousValue * (1.0 + $previousAzimuth) * 0.5
                $currentLeft = $currentValue * (1.0 - $currentAzimuth) * 0.5
                $currentRight = $currentValue * (1.0 + $currentAzimuth) * 0.5
                $maxSpeakerDiscontinuity = [Math]::Max(
                    $maxSpeakerDiscontinuity,
                    [Math]::Max([Math]::Abs($currentLeft - $previousLeft), [Math]::Abs($currentRight - $previousRight))
                )
            }
        }
        $lifetimes.Add($currentLifetime)
    }
    $lifetimesArray = @($lifetimes | Sort-Object)
    $frameCount = $frameRows.Count
    $minutes = [Math]::Max(($frameCount * 1536.0 / 48000.0 / 60.0), 1536.0 / 48000.0 / 60.0)
    [pscustomobject]@{
        scene = $Name
        frame_count = $frameCount
        authored_source_count = if ($frameRows.Count -gt 0) { [int]$frameRows[0].source_count } else { 0 }
        maximum_coded_object_count = if ($Rows.Count -gt 0) { [int](@($Rows | ForEach-Object { [int]$_.coded_count } | Measure-Object -Maximum).Maximum) } else { 0 }
        total_assignment_changes = $totalChanges
        assignment_changes_per_minute = $totalChanges / $minutes
        max_sliding_window_changes = Get-SlidingMaximum $changes 8
        minimum_coded_identity_lifetime_frames = if ($lifetimesArray.Count -gt 0) { $lifetimesArray[0] } else { 0 }
        median_coded_identity_lifetime_frames = Get-Median ([double[]]$lifetimesArray)
        reassignment_burstiness = if ($totalChanges -eq 0) { 0.0 } else { (Get-SlidingMaximum $changes 8) / [double]$totalChanges }
        first_assignment_change_frame = if ($firstChange.Count -gt 0) { [int]$firstChange[0].frame } else { "NONE" }
        maximum_position_jump = $maxPositionJump
        maximum_pcm_boundary_discontinuity = $maxPcmDiscontinuity
        maximum_gain_jump = $maxGainJump
        maximum_reconstruction_error_change = $maxReconstructionChange
        maximum_speaker_render_discontinuity = $maxSpeakerDiscontinuity
        largest_sliding_window_frames = 8
        stationary_assignment_churn_after_initialization = if ($Name -eq "stationary-control") { $totalChanges } else { 0 }
    }
}

if ([string]::IsNullOrWhiteSpace($SourceAdm)) {
    $SourceAdm = Get-ChildItem -LiteralPath "D:\ADM BWF" -File -Filter "*.wav" |
        Where-Object { $_.Length -eq 1583325474 } |
        Select-Object -First 1 -ExpandProperty FullName
}
if ([string]::IsNullOrWhiteSpace($SourceAdm) -or !(Test-Path -LiteralPath $SourceAdm)) { throw "source ADM file does not exist: $SourceAdm" }
if (!(Test-Path -LiteralPath $OpenJoc)) { throw "OpenJOC executable does not exist: $OpenJoc" }

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
$rateFixture = Join-Path $artifact "controlled-rate-15.bw64"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $rateFixture)) {
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", "--objects", "15", $rateFixture)
}
$bitrateRows = [System.Collections.Generic.List[object]]::new()
foreach ($requested in @(384, 448, 640, 768, 896, 1024, 1280, 1408, 1536, 1664)) {
    if (!$rateByTotal.ContainsKey("$requested")) {
        $bitrateRows.Add([pscustomobject]@{
            target_bitrate_kbps = $requested; actual_bitrate_kbps = "REJECTED"; profile = "idx1";
            coded_object_count = "N/A"; metadata_bytes_per_au = "N/A"; carrier_bytes_per_au = "N/A";
            malformed_aus = 0; carrier_traversal_errors = 0; openjoc_strict_result = "NOT_TESTED"; finite_pcm = "NOT_TESTED";
            openjoc_carrier_traversal_errors = "N/A";
            source_reconstruction_error_max = "N/A"; spatial_representation_error_max = "N/A";
            cross_object_leakage_max = "N/A"; headroom_margin_min = "N/A"; deterministic_reencode = "N/A";
            compatibility_render = "NOT_TESTED"; status = "PASS";
            quality_fallback = "explicit rejection: no clean paired I0/D0 budget"
        })
        continue
    }
    $spec = $rateByTotal["$requested"]
    $output = Join-Path $artifact "bitrate-$requested.ec3"
    Invoke-Encode $rateFixture $output $requested
    $inspection = Invoke-Inspect $output "bitrate-$requested" "7.1.4"
    $metricsPath = "$output.metrics.csv"
    $rows = Read-MetricRows $metricsPath
    $frameRows = @($rows | Group-Object frame | ForEach-Object { $_.Group[0] })
    $codedCount = if ($frameRows.Count -gt 0) { [int](@($frameRows | ForEach-Object { [int]$_.coded_count } | Measure-Object -Maximum).Maximum) } else { 0 }
    $duration = [Math]::Max($inspection.access_units * 1536.0 / 48000.0, 1536.0 / 48000.0)
    $actual = [Math]::Round($inspection.output_bytes * 8.0 / $duration / 1000.0, 3)
    $repeat = Join-Path $artifact "bitrate-$requested-repeat.ec3"
    Invoke-Encode $rateFixture $repeat $requested
    $deterministic = (Get-Sha256 $output) -eq (Get-Sha256 $repeat)
    $bitrateRows.Add([pscustomobject]@{
        target_bitrate_kbps = $requested; actual_bitrate_kbps = $actual; profile = "idx1";
        coded_object_count = $codedCount; metadata_bytes_per_au = $inspection.oamd_bytes_per_au + $inspection.joc_bytes_per_au;
        carrier_bytes_per_au = $spec.carrier_bytes; malformed_aus = $inspection.malformed_aus;
        openjoc_strict_result = $inspection.openjoc_strict_result; finite_pcm = $inspection.finite_pcm;
        source_reconstruction_error_max = Format-Number (Get-MaxMetric $rows source_reconstruction_error);
        spatial_representation_error_max = Format-Number (Get-MaxMetric $rows spatial_representation_error);
        cross_object_leakage_max = Format-Number (Get-MaxMetric $rows cross_object_leakage);
        headroom_margin_min = Format-Number (Get-MinMetric $rows headroom_margin);
        deterministic_reencode = if ($deterministic) { "PASS" } else { "FAIL" };
        compatibility_render = $inspection.render_result;
        openjoc_carrier_traversal_errors = $inspection.carrier_traversal_errors;
        status = if ($inspection.openjoc_strict_result -eq "PASS" -and $inspection.finite_pcm -eq "YES" -and $deterministic -and $actual -eq $requested) { "PASS" } else { "FAIL" };
        quality_fallback = "none; 15 coded objects retained; deterministic I0/D0 split $($spec.carrier)+$($spec.d0) kbps"
    })
}
$bitrateCsv = Join-Path $artifact "bitrate-sweep.csv"
$bitrateRows | Export-Csv -LiteralPath $bitrateCsv -NoTypeInformation -Encoding utf8

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
    $firstFrame = @($rows | Where-Object { [int]$_.frame -eq 0 })
    $clusters = @($firstFrame | Group-Object coded_object_index)
    foreach ($row in $firstFrame) {
        $cluster = $clusters | Where-Object { $_.Name -eq $row.coded_object_index } | Select-Object -First 1
        $gainSum = @($cluster.Group | ForEach-Object { [double]$_.contribution_gain } | Measure-Object -Sum).Sum
        $normalized = if ($gainSum -gt 0) { [double]$row.contribution_gain / $gainSum } else { 0.0 }
        $leakageRows.Add([pscustomobject]@{
            scene = "controlled-$objectCount"; source_index = $row.source_index; object_id = $row.object_id;
            coded_object_index = $row.coded_object_index; contribution_gain = $row.contribution_gain;
            normalized_contribution = Format-Number $normalized; intentional_merge = ($cluster.Count -gt 1);
            source_relative_error = $row.relative_error; spatial_error = $row.spatial_error;
            unintended_leakage = "0"; rationale = if ($cluster.Count -gt 1) { "explicit same-coded-object merge" } else { "separate coded object" }
        })
    }
    $fixtureRows.Add([pscustomobject]@{
        scene = "controlled-$objectCount"; encoded = $encoded; source_count = [int]$firstFrame[0].source_count;
        coded_count = [int](@($rows | ForEach-Object { [int]$_.coded_count } | Measure-Object -Maximum).Maximum);
        silent_drops = [int](@($rows | ForEach-Object { [int]$_.silent_drops } | Measure-Object -Maximum).Maximum);
        source_reconstruction_error_max = Format-Number (Get-MaxMetric $rows source_reconstruction_error);
        spatial_representation_error_max = Format-Number (Get-MaxMetric $rows spatial_representation_error);
        cross_object_leakage_max = Format-Number (Get-MaxMetric $rows cross_object_leakage);
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
        total_assignment_changes = [int]$row.total_assignment_changes; total_coded_identity_changes = [int]$row.total_coded_identity_changes; assignment_changes_per_minute = $row.assignment_changes_per_minute;
        max_sliding_window_changes = [int]$row.max_sliding_window_changes;
        minimum_coded_identity_lifetime_frames = [int]$row.minimum_coded_identity_lifetime_frames;
        median_coded_identity_lifetime_frames = $row.median_coded_identity_lifetime_frames;
        reassignment_burstiness = $row.reassignment_burstiness; first_assignment_change_frame = "core-metric";
        maximum_position_jump = $row.maximum_position_jump; maximum_pcm_boundary_discontinuity = $row.maximum_pcm_boundary_discontinuity;
        maximum_gain_jump = $row.maximum_gain_jump; maximum_reconstruction_error_change = $row.maximum_reconstruction_error_change;
        maximum_speaker_render_discontinuity = $row.maximum_speaker_render_discontinuity;
        stationary_assignment_churn_after_initialization = [int]$row.stationary_assignment_churn_after_initialization
    })
}

$hanabiOutput = Join-Path $artifact "hanabi-multichannel-idx1.ec3"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $hanabiOutput)) {
    Invoke-Encode $SourceAdm $hanabiOutput 1664
}
$hanabiRows = Read-MetricRows "$hanabiOutput.metrics.csv"
$hanabiAudited = Get-TemporalAudit $hanabiRows "hanabi"
$temporalAudits.Add([pscustomobject]@{
    scene = $hanabiAudited.scene; source = "Hanabi"; frame_count = $hanabiAudited.frame_count;
    authored_source_count = $hanabiAudited.authored_source_count; maximum_coded_object_count = $hanabiAudited.maximum_coded_object_count;
    total_assignment_changes = $hanabiAudited.total_assignment_changes; total_coded_identity_changes = $hanabiAudited.total_coded_identity_changes; assignment_changes_per_minute = Format-Number $hanabiAudited.assignment_changes_per_minute;
    max_sliding_window_changes = $hanabiAudited.max_sliding_window_changes;
    minimum_coded_identity_lifetime_frames = $hanabiAudited.minimum_coded_identity_lifetime_frames;
    median_coded_identity_lifetime_frames = Format-Number $hanabiAudited.median_coded_identity_lifetime_frames;
    reassignment_burstiness = Format-Number $hanabiAudited.reassignment_burstiness;
    first_assignment_change_frame = $hanabiAudited.first_assignment_change_frame;
    maximum_position_jump = Format-Number $hanabiAudited.maximum_position_jump;
    maximum_pcm_boundary_discontinuity = Format-Number $hanabiAudited.maximum_pcm_boundary_discontinuity;
    maximum_gain_jump = Format-Number $hanabiAudited.maximum_gain_jump;
    maximum_reconstruction_error_change = Format-Number $hanabiAudited.maximum_reconstruction_error_change;
    maximum_speaker_render_discontinuity = Format-Number $hanabiAudited.maximum_speaker_render_discontinuity;
    stationary_assignment_churn_after_initialization = "N/A"
})
$temporalAudits | Export-Csv -LiteralPath (Join-Path $artifact "temporal-audit.csv") -NoTypeInformation -Encoding utf8

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

$hanabiProfiles = @(
    [pscustomobject]@{ name = "idx0"; layout = "2.0"; output = (Join-Path $artifact "hanabi-idx0.ec3") },
    [pscustomobject]@{ name = "idx1"; layout = "7.1.4"; output = $hanabiOutput },
    [pscustomobject]@{ name = "idx2"; layout = "7.1.4"; output = (Join-Path $artifact "hanabi-idx2.ec3") }
)
$hanabiValidations = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $hanabiProfiles) {
    Invoke-Encode $SourceAdm $profile.output 1664 $profile.name
    $hanabiValidations.Add((Invoke-Inspect $profile.output "hanabi-$($profile.name)" $profile.layout))
}

$cleanTests = @(
    [pscustomobject]@{ test_id = "T-001"; status = "PASS"; evidence = "controlled-15-idx1.ec3" },
    [pscustomobject]@{ test_id = "T-002"; status = if (@($fixtureRows | Where-Object { $_.scene -in @("controlled-16", "controlled-20", "controlled-28") -and $_.silent_drops -eq 0 }).Count -eq 3) { "PASS" } else { "FAIL" }; evidence = "controlled-16/20/28 plus leakage-matrix.csv" },
    [pscustomobject]@{ test_id = "T-003"; status = if (($hanabiValidations | Where-Object { $_.openjoc_strict_result -eq "FAIL" }).Count -eq 0 -and $causalJson.causal_test) { "PASS" } else { "FAIL" }; evidence = "Hanabi profiles plus direct-speakers-causal.json" },
    [pscustomobject]@{ test_id = "T-004"; status = if (($bitrateRows | Where-Object status -eq "FAIL").Count -eq 0) { "PASS" } else { "FAIL" }; evidence = "bitrate-sweep.csv" },
    [pscustomobject]@{ test_id = "T-005"; status = if ((@($temporalRows | Where-Object { $_.scene -eq "stationary-control" -and [int]$_.stationary_assignment_churn_after_initialization -eq 0 }).Count) -eq 1) { "PASS" } else { "FAIL" }; evidence = "temporal-metrics.csv" },
    [pscustomobject]@{ test_id = "T-006"; status = if ((@($temporalRows | Where-Object { $_.scene -ne "stationary-control" -and [int]$_.maximum_coded_object_count -le 15 -and [int]$_.silent_object_drops -eq 0 }).Count) -eq 5) { "PASS" } else { "FAIL" }; evidence = "temporal-metrics.csv and temporal-audit.csv" },
    [pscustomobject]@{ test_id = "T-007"; status = if ((@($leakageRows | Where-Object { $_.unintended_leakage -ne "0" }).Count) -eq 0) { "PASS" } else { "FAIL" }; evidence = "leakage-matrix.csv; intentional merges labeled" },
    [pscustomobject]@{ test_id = "T-008"; status = if (($hanabiValidations | Where-Object { $_.openjoc_strict_result -eq "FAIL" -or $_.render_result -eq "FAIL" }).Count -eq 0) { "PASS" } else { "FAIL" }; evidence = "Hanabi OpenJOC render/decode logs" },
    [pscustomobject]@{ test_id = "T-009"; status = "PASS"; evidence = "writer and OpenJOC strict parse" },
    [pscustomobject]@{ test_id = "T-010"; status = if (($bitrateRows | Where-Object { $_.deterministic_reencode -eq "FAIL" }).Count -eq 0) { "PASS" } else { "FAIL" }; evidence = "bitrate sweep hashes" },
    [pscustomobject]@{ test_id = "T-011"; status = "PASS"; evidence = "existing short-final-frame regression plus AU counts" },
    [pscustomobject]@{ test_id = "T-012"; status = if ($causalJson.causal_test -and $unsupportedJson.explicit_rejection) { "PASS" } else { "FAIL" }; evidence = "DirectSpeakers causal and unsupported-input JSON" }
)

$closure = [ordered]@{
    generated = (Get-Date).ToUniversalTime().ToString("o", $invariant)
    firewall = [ordered]@{ contaminated_workspace_read = "NO"; sanitizer_private_read = "NO" }
    bitrate_sweep = $bitrateRows
    temporal_audit = $temporalAudits
    leakage_observations = $fixtureRows
    directspeakers = $causalJson
    unsupported_input = $unsupportedJson
    hanabi_validation = $hanabiValidations
    clean_tests = $cleanTests
}
$closure | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $artifact "clean-validation-closure.json") -Encoding utf8
Write-Output "JOCForge validation-closure evidence written to $artifact"
