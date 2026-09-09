# pattern: Imperative Shell

param(
    [string]$SourceAdm = "",
    [switch]$ReuseExisting
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\temporal-stabilization"
New-Item -ItemType Directory -Force $artifact | Out-Null
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$OpenJoc = (Resolve-OpenJocValidator $root).executable

function Invoke-Checked([string]$File, [string[]]$Arguments) {
    & $File @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed with exit code $LASTEXITCODE`: $File $($Arguments -join ' ')"
    }
}

function Get-Sha256([string]$Path) {
    $sha = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($Path)
    try { return (($sha.ComputeHash($stream) | ForEach-Object { $_.ToString("x2") }) -join "") }
    finally { $stream.Dispose(); $sha.Dispose() }
}

function Invoke-Encode([string]$InputPath, [string]$OutputPath, [string]$Profile, [double]$TemporalWeight) {
    $metrics = "$OutputPath.metrics.csv"
    if ($ReuseExisting -and (Test-Path -LiteralPath $OutputPath) -and (Test-Path -LiteralPath $metrics) -and
        (Get-Content -LiteralPath $metrics -TotalCount 1) -match "authored_source_assignment_changes,representative_source_changes") { return }
    Invoke-Checked "cargo" @(
        "run", "-p", "jocforge", "--quiet", "--", "encode", $InputPath, $OutputPath,
        "--profile", $Profile, "--bitrate", "1664", "--temporal-weight", "$TemporalWeight"
    )
}

function Get-Median([double[]]$Values) {
    if ($Values.Count -eq 0) { return 0.0 }
    $ordered = @($Values | Sort-Object)
    $middle = [int]($ordered.Count / 2)
    if ($ordered.Count % 2 -eq 0) { return ($ordered[$middle - 1] + $ordered[$middle]) / 2.0 }
    $ordered[$middle]
}

function Get-SlidingMaximum([int[]]$Values, [int]$Window) {
    $maximum = 0
    for ($index = 0; $index -lt $Values.Count; $index++) {
        $sum = 0
        for ($cursor = [Math]::Max(0, $index - $Window + 1); $cursor -le $index; $cursor++) { $sum += $Values[$cursor] }
        $maximum = [Math]::Max($maximum, $sum)
    }
    $maximum
}

function Get-RunLifetimes($Values) {
    $Values = @($Values)
    if ($Values.Count -eq 0) { return @() }
    $lifetimes = [System.Collections.Generic.List[int]]::new()
    $current = $Values[0]
    $length = 1
    foreach ($value in ($Values | Select-Object -Skip 1)) {
        if ($value -eq $current) { $length++ }
        else { $lifetimes.Add($length); $current = $value; $length = 1 }
    }
    $lifetimes.Add($length)
    @($lifetimes)
}

function Get-Audit([string]$MetricsPath, [string]$Name) {
    $rows = @(Import-Csv -LiteralPath $MetricsPath | Where-Object record -eq source)
    $frameGroups = @($rows | Group-Object frame | Sort-Object { [int]$_.Name })
    $frameRows = @($frameGroups | ForEach-Object { $_.Group[0] })
    $changes = @($frameRows | Select-Object -Skip 1 | ForEach-Object { [int]$_.authored_source_assignment_changes })
    $identityChanges = @($frameRows | Select-Object -Skip 1 | ForEach-Object { [int]$_.representative_source_changes })
    $totalChanges = if ($changes.Count) { ($changes | Measure-Object -Sum).Sum } else { 0 }
    $totalIdentityChanges = if ($identityChanges.Count) { ($identityChanges | Measure-Object -Sum).Sum } else { 0 }
    $slotReplacementChanges = [System.Collections.Generic.List[int]]::new()
    for ($frameIndex = 1; $frameIndex -lt $frameGroups.Count; $frameIndex++) {
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
        $slotReplacementChanges.Add(@($previousSlots.Keys | Where-Object { $currentSlots.ContainsKey($_) -and $previousSlots[$_] -ne $currentSlots[$_] }).Count)
    }
    $totalSlotReplacementChanges = if ($slotReplacementChanges.Count) { ($slotReplacementChanges | Measure-Object -Sum).Sum } else { 0 }

    $assignmentLifetimes = [System.Collections.Generic.List[int]]::new()
    foreach ($group in @($rows | Group-Object source_index)) {
        $ordered = @($group.Group | Sort-Object { [int]$_.frame })
        $current = [int]$ordered[0].coded_object_index
        $lifetime = 1
        for ($index = 1; $index -lt $ordered.Count; $index++) {
            if ([int]$ordered[$index].coded_object_index -eq $current) { $lifetime++ }
            else { $assignmentLifetimes.Add($lifetime); $current = [int]$ordered[$index].coded_object_index; $lifetime = 1 }
        }
        $assignmentLifetimes.Add($lifetime)
    }

    $representativeSourceLifetimes = [System.Collections.Generic.List[int]]::new()
    foreach ($group in @($rows | Group-Object coded_representative_source_index)) {
        $frames = @($group.Group | ForEach-Object { [int]$_.frame } | Sort-Object -Unique)
        if ($frames.Count -eq 0) { continue }
        $lifetime = 1
        for ($index = 1; $index -lt $frames.Count; $index++) {
            if ($frames[$index] -eq $frames[$index - 1] + 1) { $lifetime++ }
            else { $representativeSourceLifetimes.Add($lifetime); $lifetime = 1 }
        }
        $representativeSourceLifetimes.Add($lifetime)
    }

    $positionJump = 0.0
    $pcmDiscontinuity = 0.0
    $gainJump = 0.0
    $speakerDiscontinuity = 0.0
    $activityEvents = 0
    $sourceMotionEvents = 0
    $largePositionJumps = 0
    $gainJumpsOverHalf = 0
    foreach ($group in @($rows | Group-Object source_index)) {
        $ordered = @($group.Group | Sort-Object { [int]$_.frame })
        for ($index = 1; $index -lt $ordered.Count; $index++) {
            $previous = $ordered[$index - 1]; $current = $ordered[$index]
            if ([int]$previous.coded_object_index -eq [int]$current.coded_object_index) { continue }
            if (([double]$previous.source_energy -lt 1e-12) -xor ([double]$current.source_energy -lt 1e-12)) { $activityEvents++ }
            $sourceMotion = [Math]::Sqrt(([double]$previous.source_azimuth - [double]$current.source_azimuth) * ([double]$previous.source_azimuth - [double]$current.source_azimuth) + ([double]$previous.source_elevation - [double]$current.source_elevation) * ([double]$previous.source_elevation - [double]$current.source_elevation) + ([double]$previous.source_distance - [double]$current.source_distance) * ([double]$previous.source_distance - [double]$current.source_distance))
            if ($sourceMotion -gt 1e-12) { $sourceMotionEvents++ }
            $jump = [Math]::Sqrt(([double]$previous.coded_azimuth - [double]$current.coded_azimuth) * ([double]$previous.coded_azimuth - [double]$current.coded_azimuth) + ([double]$previous.coded_elevation - [double]$current.coded_elevation) * ([double]$previous.coded_elevation - [double]$current.coded_elevation) + ([double]$previous.coded_distance - [double]$current.coded_distance) * ([double]$previous.coded_distance - [double]$current.coded_distance))
            $positionJump = [Math]::Max($positionJump, $jump); if ($jump -gt 0.5) { $largePositionJumps++ }
            $pcmDiscontinuity = [Math]::Max($pcmDiscontinuity, [Math]::Abs([double]$previous.coded_last_sample - [double]$current.coded_first_sample))
            $gain = [Math]::Abs([double]$previous.contribution_gain - [double]$current.contribution_gain)
            $gainJump = [Math]::Max($gainJump, $gain); if ($gain -gt 0.5) { $gainJumpsOverHalf++ }
            $previousValue = [double]$previous.coded_last_sample; $currentValue = [double]$current.coded_first_sample
            $previousAzimuth = [Math]::Max(-1.0, [Math]::Min(1.0, [double]$previous.coded_azimuth)); $currentAzimuth = [Math]::Max(-1.0, [Math]::Min(1.0, [double]$current.coded_azimuth))
            $previousLeft = $previousValue * (1.0 - $previousAzimuth) * 0.5; $previousRight = $previousValue * (1.0 + $previousAzimuth) * 0.5
            $currentLeft = $currentValue * (1.0 - $currentAzimuth) * 0.5; $currentRight = $currentValue * (1.0 + $currentAzimuth) * 0.5
            $speakerDiscontinuity = [Math]::Max($speakerDiscontinuity, [Math]::Max([Math]::Abs($currentLeft - $previousLeft), [Math]::Abs($currentRight - $previousRight)))
        }
    }

    $minutes = [Math]::Max($frameRows.Count * 1536.0 / 48000.0 / 60.0, 0.0000001)
    $reconstruction = @($rows | ForEach-Object { [double]$_.source_reconstruction_error })
    $spatial = @($rows | ForEach-Object { [double]$_.spatial_representation_error })
    $leakage = @($rows | ForEach-Object { [double]$_.cross_object_leakage })
    $representativeSource = @($representativeSourceLifetimes | ForEach-Object { [double]$_ })
    $assignment = @($assignmentLifetimes | ForEach-Object { [double]$_ })
    $maximumCodedObjectCount = if ($rows.Count) { (@($rows | ForEach-Object { [int]$_.coded_count } | Measure-Object -Maximum).Maximum) } else { 0 }
    $codedSlotLifetimes = [System.Collections.Generic.List[int]]::new()
    for ($slot = 0; $slot -lt $maximumCodedObjectCount; $slot++) {
        $presence = @($frameRows | ForEach-Object { [int]$_.coded_count -gt $slot })
        foreach ($lifetime in @(Get-RunLifetimes $presence)) {
            $codedSlotLifetimes.Add([int]$lifetime)
        }
    }
    $slotLifetimeArray = @($codedSlotLifetimes | Sort-Object | ForEach-Object { [double]$_ })
    [pscustomobject]@{
        name = $Name; frame_count = $frameRows.Count; authored_source_count = if ($frameRows.Count) { [int]$frameRows[0].source_count } else { 0 }
        maximum_coded_object_count = $maximumCodedObjectCount
        total_authored_source_assignment_changes = $totalChanges; total_representative_source_changes = $totalIdentityChanges; total_coded_slot_replacements = $totalSlotReplacementChanges
        authored_source_assignment_changes_per_minute = $totalChanges / $minutes; max_sliding_window_authored_source_assignment_changes = Get-SlidingMaximum $changes 8
        minimum_authored_source_assignment_lifetime_frames = if ($assignment.Count) { (@($assignment | Measure-Object -Minimum).Minimum) } else { 0 }
        median_authored_source_assignment_lifetime_frames = Get-Median ([double[]]$assignment)
        minimum_representative_source_lifetime_frames = if ($representativeSource.Count) { (@($representativeSource | Measure-Object -Minimum).Minimum) } else { 0 }
        median_representative_source_lifetime_frames = Get-Median ([double[]]$representativeSource)
        minimum_coded_slot_lifetime_frames = if ($slotLifetimeArray.Count) { $slotLifetimeArray[0] } else { 0 }
        median_coded_slot_lifetime_frames = Get-Median ([double[]]$slotLifetimeArray)
        authored_source_assignment_reassignment_burstiness = if ($totalChanges) { (Get-SlidingMaximum $changes 8) / [double]$totalChanges } else { 0.0 }
        maximum_position_jump = $positionJump; maximum_pcm_boundary_discontinuity = $pcmDiscontinuity; maximum_gain_jump = $gainJump
        maximum_speaker_render_discontinuity = $speakerDiscontinuity; activity_boundary_events = $activityEvents; source_motion_events = $sourceMotionEvents
        large_position_jumps = $largePositionJumps; gain_jumps_over_half = $gainJumpsOverHalf
        maximum_source_reconstruction_error = if ($reconstruction.Count) { (@($reconstruction | Measure-Object -Maximum).Maximum) } else { 0.0 }
        maximum_spatial_representation_error = if ($spatial.Count) { (@($spatial | Measure-Object -Maximum).Maximum) } else { 0.0 }
        maximum_cross_object_leakage = if ($leakage.Count) { (@($leakage | Measure-Object -Maximum).Maximum) } else { 0.0 }
    }
}

function Invoke-Validation([string]$Path, [string]$Name) {
    $inspect = Join-Path $artifact "$Name.openjoc-inspect.log"
    $decode = Join-Path $artifact "$Name.ffmpeg.log"
    & $OpenJoc inspect $Path *> $inspect; $inspectExit = $LASTEXITCODE
    $priorErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
    & ffmpeg -hide_banner -v error -f eac3 -i $Path -f null - *> $decode
        $decodeExit = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $priorErrorActionPreference
    }
    $text = Get-Content -LiteralPath $inspect -Raw
    $malformed = if ($text -match "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed") { [int]$Matches[1] } else { -1 }
    [pscustomobject]@{
        name = $Name; openjoc_strict = ($inspectExit -eq 0 -and $malformed -eq 0 -and $text -notmatch "profile candidate parsing failed|decode error")
        malformed_aus = $malformed
        ffmpeg_decode = ($decodeExit -eq 0); finite_pcm = ($decodeExit -eq 0); channels = $null
    }
}

if ([string]::IsNullOrWhiteSpace($SourceAdm)) {
    throw "source ADM file is required: pass -SourceAdm <local ADM-BWF path>; copyrighted source media is not included"
}
if (!(Test-Path -LiteralPath $SourceAdm -PathType Leaf)) { throw "source ADM file does not exist: $SourceAdm" }
$validatorIdentity = Assert-OpenJocValidatorPin $root $OpenJoc
$validatorIdentity | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "openjoc-validator-identity.json") -Encoding utf8

$beforePath = Join-Path $artifact "hanabi-before.ec3"
$afterPath = Join-Path $artifact "hanabi-after.ec3"
Invoke-Encode $SourceAdm $beforePath "idx1" 0.0
Invoke-Encode $SourceAdm $afterPath "idx1" 0.15
$beforeAudit = Get-Audit "$beforePath.metrics.csv" "before"
$afterAudit = Get-Audit "$afterPath.metrics.csv" "after"

$adversarialBefore = Join-Path $artifact "adversarial-before.csv"
$adversarialAfter = Join-Path $artifact "adversarial-after.csv"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $adversarialBefore)) { Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "temporal-adversarial", "--no-continuity", $adversarialBefore) }
if (!$ReuseExisting -or !(Test-Path -LiteralPath $adversarialAfter)) { Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "temporal-adversarial", $adversarialAfter) }
$adversarialBeforeRows = @(Import-Csv $adversarialBefore); $adversarialAfterRows = @(Import-Csv $adversarialAfter)

$profiles = @(
    [pscustomobject]@{ name = "idx0"; output = (Join-Path $artifact "hanabi-idx0.ec3") },
    [pscustomobject]@{ name = "idx1"; output = $afterPath },
    [pscustomobject]@{ name = "idx2"; output = (Join-Path $artifact "hanabi-idx2.ec3") }
)
$profileValidation = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $profiles) {
    if ($profile.name -ne "idx1") {
        Invoke-Encode $SourceAdm $profile.output $profile.name 0.15
    }
    $profileValidation.Add((Invoke-Validation $profile.output "after-$($profile.name)"))
}
$repeat = Join-Path $artifact "hanabi-after-repeat.ec3"
Invoke-Encode $SourceAdm $repeat "idx1" 0.15
$deterministic = (Get-Sha256 $afterPath) -eq (Get-Sha256 $repeat)

$comparison = [ordered]@{
    metric_valid = $true
    root_cause = "frame-local source-to-seed membership winner changes under signal/activity cost; previous-seed reuse keeps representative identities stable"
    before = $beforeAudit
    after = $afterAudit
    improvement = [ordered]@{
        authored_source_assignment_change_reduction_ratio = if ($beforeAudit.total_authored_source_assignment_changes) { 1.0 - $afterAudit.total_authored_source_assignment_changes / [double]$beforeAudit.total_authored_source_assignment_changes } else { 1.0 }
        max_sliding_window_authored_source_assignment_reduction_ratio = if ($beforeAudit.max_sliding_window_authored_source_assignment_changes) { 1.0 - $afterAudit.max_sliding_window_authored_source_assignment_changes / [double]$beforeAudit.max_sliding_window_authored_source_assignment_changes } else { 1.0 }
        median_authored_source_assignment_lifetime_delta_frames = $afterAudit.median_authored_source_assignment_lifetime_frames - $beforeAudit.median_authored_source_assignment_lifetime_frames
        representative_source_changes_before = $beforeAudit.total_representative_source_changes
        representative_source_changes_after = $afterAudit.total_representative_source_changes
    }
    quality_regression = [ordered]@{
        reconstruction = ($afterAudit.maximum_source_reconstruction_error -le $beforeAudit.maximum_source_reconstruction_error)
        spatial = ($afterAudit.maximum_spatial_representation_error -le $beforeAudit.maximum_spatial_representation_error)
        leakage = ($afterAudit.maximum_cross_object_leakage -le $beforeAudit.maximum_cross_object_leakage)
    }
    adversarial_before = $adversarialBeforeRows
    adversarial_after = $adversarialAfterRows
    profile_validation = $profileValidation
    deterministic_reencode = $deterministic
    artifacts = [ordered]@{ before_after = (Join-Path $artifact "hanabi-before-after.json"); adversarial_before = $adversarialBefore; adversarial_after = $adversarialAfter }
}
$comparison | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $artifact "hanabi-before-after.json") -Encoding utf8
Write-Output "JOCForge temporal stabilization evidence written to $artifact"
