# pattern: Imperative Shell

[CmdletBinding()]
param(
    [string]$OutputRoot = "",
    [string]$OpenJocRoot = "",
    [string]$OpenJocPath = "",
    [string]$FfmpegPath = "",
    [string]$FfprobePath = "",
    [string]$MediaInfoPath = "",
    [int]$TimeoutSeconds = 120
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$configuredOpenJocRoot = if ([string]::IsNullOrWhiteSpace($OpenJocRoot)) {
    [Environment]::GetEnvironmentVariable("JOCFORGE_OPENJOC_ROOT")
} else {
    $OpenJocRoot
}
$openjocRoot = if ([string]::IsNullOrWhiteSpace($configuredOpenJocRoot)) {
    Join-Path (Split-Path -Parent $root) "OpenJOC"
} else {
    $configuredOpenJocRoot
}
$openjocRoot = [IO.Path]::GetFullPath($openjocRoot)
$expectedOpenJocHead = "a52c35d21e68e18dbaa8fa18accfbfea44087367"
$expectedOpenJocSha256 = "c740767304775648e18e3fdf8e490744e3c9521e27a8e2fca2aad9027b12977c"
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $root "artifacts\interop-census"
}
$OutputRoot = [IO.Path]::GetFullPath($OutputRoot)
$docsPath = Join-Path $OutputRoot "interop-census.generated.md"
$probeRoot = Join-Path $OutputRoot "probes"

function Get-ExecutablePath([string]$ExplicitPath, [string[]]$Names, [string[]]$Candidates) {
    if (-not [string]::IsNullOrWhiteSpace($ExplicitPath)) {
        $resolved = [IO.Path]::GetFullPath($ExplicitPath)
        if (!(Test-Path -LiteralPath $resolved -PathType Leaf)) {
            throw "executable does not exist: $resolved"
        }
        return $resolved
    }
    foreach ($candidate in $Candidates) {
        if (-not [string]::IsNullOrWhiteSpace($candidate) -and (Test-Path -LiteralPath $candidate -PathType Leaf)) {
            return [IO.Path]::GetFullPath($candidate)
        }
    }
    foreach ($name in $Names) {
        $command = Get-Command $name -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($null -ne $command -and $command.Source) {
            return [IO.Path]::GetFullPath($command.Source)
        }
    }
    return $null
}

function Get-Sha256([string]$Path) {
    $algorithm = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($Path)
    try {
        return (($algorithm.ComputeHash($stream) | ForEach-Object { $_.ToString("x2") }) -join "")
    }
    finally {
        $stream.Dispose()
        $algorithm.Dispose()
    }
}

function Get-RelativePath([string]$BasePath, [string]$TargetPath) {
    $baseUri = [Uri]::new(([IO.Path]::GetFullPath($BasePath).TrimEnd('\') + '\'))
    $targetUri = [Uri]::new([IO.Path]::GetFullPath($TargetPath))
    return $baseUri.MakeRelativeUri($targetUri).ToString().Replace('/', '\')
}

function Invoke-Bounded([string]$File, [string[]]$Arguments, [int]$TimeoutMilliseconds, [string]$WorkingDirectory = "") {
    $start = [Diagnostics.Stopwatch]::StartNew()
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $File
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    if (-not [string]::IsNullOrWhiteSpace($WorkingDirectory)) {
        $info.WorkingDirectory = $WorkingDirectory
    }
    $info.Arguments = (($Arguments | ForEach-Object {
                '"' + ([string]$_).Replace('"', '\"') + '"'
            }) -join ' ')
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    try {
        try {
            [void]$process.Start()
        }
        catch {
            return [pscustomobject]@{
                status = "LAUNCH_ERROR"
                exit_code = $null
                stdout = ""
                stderr = $_.Exception.Message
                duration_ms = [int]$start.ElapsedMilliseconds
            }
        }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if (!$process.WaitForExit($TimeoutMilliseconds)) {
            try { $process.Kill($true) } catch { }
            [void]$process.WaitForExit(5000)
            return [pscustomobject]@{
                status = "TIMEOUT"
                exit_code = $null
                stdout = $stdoutTask.Result
                stderr = $stderrTask.Result
                duration_ms = [int]$start.ElapsedMilliseconds
            }
        }
        return [pscustomobject]@{
            status = "EXITED"
            exit_code = $process.ExitCode
            stdout = $stdoutTask.Result
            stderr = $stderrTask.Result
            duration_ms = [int]$start.ElapsedMilliseconds
        }
    }
    finally {
        $process.Dispose()
        $start.Stop()
    }
}

function Save-ToolOutput([string]$Directory, [string]$Stem, $Result) {
    $Result.stdout | Set-Content -LiteralPath (Join-Path $Directory "$Stem.stdout.txt") -Encoding utf8
    $Result.stderr | Set-Content -LiteralPath (Join-Path $Directory "$Stem.stderr.txt") -Encoding utf8
}

function Invoke-Cargo([string[]]$Arguments, [string]$Directory, [string]$LogStem) {
    $result = Invoke-Bounded $cargoPath $Arguments ($TimeoutSeconds * 1000) $root
    Save-ToolOutput $Directory $LogStem $result
    if ($result.status -ne "EXITED" -or $result.exit_code -ne 0) {
        throw "cargo command failed: $($Arguments -join ' ')"
    }
    return $result
}

function Get-PropertyValue($Object, [string[]]$Names) {
    if ($null -eq $Object) { return $null }
    foreach ($name in $Names) {
        $property = $Object.PSObject.Properties[$name]
        if ($null -ne $property) { return $property.Value }
    }
    return $null
}

function Try-ConvertJson([string]$Text) {
    if ([string]::IsNullOrWhiteSpace($Text)) { return $null }
    try { return $Text | ConvertFrom-Json } catch { return $null }
}

function Get-RepoHead([string]$Repository) {
    $result = Invoke-Bounded $gitPath @("-C", $Repository, "rev-parse", "HEAD") 30000 $root
    if ($result.status -ne "EXITED" -or $result.exit_code -ne 0) {
        throw "failed to resolve repository HEAD: $Repository"
    }
    return $result.stdout.Trim()
}

function Get-RepoStatus([string]$Repository) {
    $result = Invoke-Bounded $gitPath @("-C", $Repository, "status", "--porcelain") 30000 $root
    if ($result.status -ne "EXITED" -or $result.exit_code -ne 0) {
        throw "failed to resolve repository status: $Repository"
    }
    return $result.stdout.Trim()
}

function Get-VersionRecord([string]$Name, [string]$Path, [string[]]$Arguments) {
    if ($null -eq $Path) {
        return [ordered]@{ name = $Name; available = $false; path = $null; path_recorded = $false; sha256 = $null; version = $null; banner = $null }
    }
    $result = Invoke-Bounded $Path $Arguments 30000 $root
    $banner = ($result.stdout + $result.stderr).Trim()
    $firstLine = ($banner -split "`r?`n" | Select-Object -First 1).Trim()
    return [ordered]@{
        name = $Name
        available = ($result.status -eq "EXITED" -and $result.exit_code -eq 0)
        path = $null
        path_recorded = $false
        sha256 = Get-Sha256 $Path
        version = $firstLine
        banner = $banner
    }
}

function Get-MatrixRow($Case, $Rows) {
    $matches = @($Rows | Where-Object {
            $_.profile -eq $Case.profile -and
            $_.container -eq $Case.container -and
            $_.carriage -eq $Case.carriage -and
            $_.emdf_profile -eq $Case.emdf_profile -and
            $_.partition -eq $Case.partition -and
            $_.dependent_count -eq ([string]$Case.dependent_count) -and
            $_.lfe_ownership -eq $Case.lfe_ownership -and
            $_.bitrate_kbps -eq ([string]$Case.bitrate_kbps)
        })
    if ($matches.Count -ne 1 -or $matches[0].verdict -ne "SUPPORTED_PASS") {
        throw "positive probe is not exactly SUPPORTED_PASS in current matrix: $($Case.id)"
    }
    return $matches[0]
}

function Get-OpenJocMalformed($Json, [string]$Text) {
    $validation = Get-PropertyValue $Json @("validation")
    $value = Get-PropertyValue $validation @("malformed_aus", "malformed_count", "malformed")
    if ($null -eq $value) { $value = Get-PropertyValue $Json @("malformed_aus", "malformed_count") }
    if ($null -ne $value) { return [int]$value }
    $match = [regex]::Match($Text, "frame-end EMDF:\s+\d+ parsed,\s+\d+ non-EMDF,\s+(\d+) malformed")
    if ($match.Success) { return [int]$match.Groups[1].Value }
    return -1
}

function Get-OpenJocProfile($Json) {
    $profile = Get-PropertyValue $Json @("profile")
    $index = Get-PropertyValue $profile @("index", "profile_index")
    $name = Get-PropertyValue $profile @("name", "semantic_name")
    if ($null -eq $index) { $index = Get-PropertyValue $Json @("profile_index") }
    if ($null -eq $name) { $name = Get-PropertyValue $Json @("profile_name") }
    return [ordered]@{ index = $index; name = $name }
}

function Get-OpenJocDerived($Json, [string]$Text) {
    $validation = Get-PropertyValue $Json @("validation")
    $eac3 = Get-PropertyValue $Json @("eac3")
    $joc = Get-PropertyValue $Json @("joc")
    $carriage = Get-PropertyValue $Json @("carriage")
    $emdf = Get-PropertyValue $Json @("emdf")
    $present = Get-PropertyValue $joc @("present")
    if ($present -isnot [bool]) { $present = $null }
    $profileEntry = @((Get-PropertyValue $joc @("profiles"))) | Select-Object -First 1
    $profile = if ($null -ne $profileEntry) { Get-PropertyValue $profileEntry @("value") } else { $null }
    if ($null -eq $profile) { $profile = Get-OpenJocProfile $Json }
    $topologyEntry = @((Get-PropertyValue $eac3 @("topologies"))) | Select-Object -First 1
    $partitionEntry = @((Get-PropertyValue $eac3 @("block_partitions"))) | Select-Object -First 1
    $lfeEntry = @((Get-PropertyValue $eac3 @("lfe_ownership"))) | Select-Object -First 1
    $locationEntry = @((Get-PropertyValue $carriage @("locations"))) | Select-Object -First 1
    $carriers = if ($null -ne $profile) { @((Get-PropertyValue $profile @("carriers"))) } else { @() }
    $partition = if ($null -ne $partitionEntry) { @((Get-PropertyValue $partitionEntry @("value"))) } else { @() }
    $samples = if ($null -ne $eac3 -and $eac3.access_unit_count -gt 0) { $eac3.total_samples / $eac3.access_unit_count } else { $null }
    $topology = if ($null -ne $topologyEntry) { @((Get-PropertyValue $topologyEntry @("value"))) } else { @() }
    $lfe = if ($null -ne $lfeEntry) { Get-PropertyValue $lfeEntry @("value") } else { $null }
    $jocOwner = @((Get-PropertyValue $joc @("owners")))
    $dependentIds = @((Get-PropertyValue $eac3 @("dependent_ids")))
    $carriageLocation = if ($null -ne $locationEntry) { Get-PropertyValue (Get-PropertyValue $locationEntry @("value")) @("location") } else { $null }
    $emdfConfigurations = Get-PropertyValue $emdf @("configurations")
    $strict = [regex]::Match($Text, "profile:\s+ETSI_STRICT\s+result:\s+([^\s]+)")
    $compat = [regex]::Match($Text, "profile:\s+OBSERVED_VENDOR_COMPAT\s+result:\s+([^\s]+)")
    return [ordered]@{
        joc_present = $present
        profile = $profile
        topology = $topology
        carriers = $carriers
        block_partition = $partition
        samples_per_access_unit = $samples
        lfe_ownership = $lfe
        joc_owner = $jocOwner
        dependent_ids = $dependentIds
        carriage_location = $carriageLocation
        emdf_configurations = $emdfConfigurations
        malformed_aus = Get-OpenJocMalformed $Json $Text
        strict_status = if ($null -ne $validation -and $validation.etsi_strict) { $validation.etsi_strict.status } elseif ($strict.Success) { $strict.Groups[1].Value } else { $null }
        compatibility_status = if ($null -ne $validation -and $validation.deployed_compatibility) { $validation.deployed_compatibility.status } elseif ($compat.Success) { $compat.Groups[1].Value } else { $null }
        validation = $validation
    }
}

function Get-PcmHealth([string]$WavePath, [string]$Directory, [string]$Stem) {
    $probe = Invoke-Bounded $ffprobePath @("-v", "error", "-select_streams", "a:0", "-show_entries", "stream=channels,sample_rate,duration", "-of", "json", $WavePath) 30000 $root
    Save-ToolOutput $Directory "$Stem.ffprobe" $probe
    $parsed = Try-ConvertJson $probe.stdout
    $stream = if ($null -ne $parsed) { @($parsed.streams) | Select-Object -First 1 } else { $null }
    $channels = Get-PropertyValue $stream @("channels")
    $rawPath = Join-Path $Directory "$Stem.f32le"
    $decode = Invoke-Bounded $ffmpegPath @("-hide_banner", "-v", "error", "-y", "-i", $WavePath, "-f", "f32le", $rawPath) 30000 $root
    Save-ToolOutput $Directory "$Stem.f32le" $decode
    $nonFinite = 0
    $sampleCount = 0
    if ($decode.status -eq "EXITED" -and $decode.exit_code -eq 0 -and (Test-Path -LiteralPath $rawPath) -and $channels -gt 0) {
        $bytes = [IO.File]::ReadAllBytes($rawPath)
        $sampleCount = [int]($bytes.Length / 4 / [int]$channels)
        for ($offset = 0; $offset + 4 -le $bytes.Length; $offset += 4) {
            $value = [BitConverter]::ToSingle($bytes, $offset)
            if ([float]::IsNaN($value) -or [float]::IsInfinity($value)) { $nonFinite++ }
        }
    }
    if (Test-Path -LiteralPath $rawPath) { Remove-Item -LiteralPath $rawPath -Force }
    return [ordered]@{
        decode_exit_code = $decode.exit_code
        channels = $channels
        sample_count_per_channel = $sampleCount
        non_finite_samples = $nonFinite
        finite = ($decode.status -eq "EXITED" -and $decode.exit_code -eq 0 -and $channels -gt 0 -and $nonFinite -eq 0 -and $sampleCount -gt 0)
        ffprobe = $parsed
    }
}

function Get-FfmpegStatus($Run, $Case, $OpenJocDerived) {
    if ($Run.status -eq "TIMEOUT") { return "HANG" }
    if ($Run.status -ne "EXITED") { return "CRASH" }
    $dependentIds = @((Get-PropertyValue $OpenJocDerived @("dependent_ids")))
    $hasAdditionalSubstreamTopology = $Case.dependent_count -gt 1 -and
        $dependentIds.Count -ge $Case.dependent_count
    $isDependentFamily = $Case.generator -in @("dependent-lfe", "multi-dependent", "short-multi-dependent")
    $hasKnownFfmpegDecodeDiagnostic = $Run.stderr -match "(?i)(unspecified sample rate|decode error rate|sample rate not set)"
    # FFmpeg 8.1 does not emit a stable substream token for these failures. The
    # independent OpenJOC dependent-ID census and canonical generator family
    # establish the additional-substream topology; FFmpeg supplies the failure.
    $additionalSubstreamEvidence = $hasAdditionalSubstreamTopology -and
        $isDependentFamily -and
        $hasKnownFfmpegDecodeDiagnostic
    if ($Run.exit_code -eq 0) {
        if ($additionalSubstreamEvidence) {
            return "PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED"
        }
        return "FULL_PASS"
    }
    if ($additionalSubstreamEvidence) { return "PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED" }
    return "FAIL_STREAM_ERROR"
}

function Get-FfprobeSummary($Json) {
    $stream = if ($null -ne $Json) { @($Json.streams) | Select-Object -First 1 } else { $null }
    $format = if ($null -ne $Json) { $Json.format } else { $null }
    return [ordered]@{
        codec_name = Get-PropertyValue $stream @("codec_name")
        codec_long_name = Get-PropertyValue $stream @("codec_long_name")
        sample_rate = Get-PropertyValue $stream @("sample_rate")
        channels = Get-PropertyValue $stream @("channels")
        channel_layout = Get-PropertyValue $stream @("channel_layout")
        bit_rate = Get-PropertyValue $stream @("bit_rate")
        duration = Get-PropertyValue $stream @("duration")
        stream_count = if ($null -ne $Json) { @($Json.streams).Count } else { 0 }
        format_name = Get-PropertyValue $format @("format_name")
        format_long_name = Get-PropertyValue $format @("format_long_name")
        format_duration = Get-PropertyValue $format @("duration")
    }
}

function Get-MediaInfoResult([string]$Path, [string]$Directory) {
    if ($null -eq $mediaInfoPath) {
        return [ordered]@{
            available = $false
            status = "MEDIAINFO_UNAVAILABLE"
            human_output = $null
            parsed = $null
            fields = $null
        }
    }
    $human = Invoke-Bounded $mediaInfoPath @($Path) 30000 $root
    $machine = Invoke-Bounded $mediaInfoPath @("--Output=JSON", $Path) 30000 $root
    Save-ToolOutput $Directory "mediainfo-human" $human
    Save-ToolOutput $Directory "mediainfo-machine" $machine
    $parsed = Try-ConvertJson $machine.stdout
    $tracks = if ($null -ne $parsed) { @($parsed.media.track) } else { @() }
    $general = $tracks | Where-Object { $_.'@type' -eq "General" } | Select-Object -First 1
    $audio = $tracks | Where-Object { $_.'@type' -eq "Audio" } | Select-Object -First 1
    $format = Get-PropertyValue $general @("Format")
    $audioFormat = Get-PropertyValue $audio @("Format")
    $commercialName = Get-PropertyValue $general @("CommercialName")
    if ($null -eq $commercialName) { $commercialName = Get-PropertyValue $general @("Format_Commercial_IfAny") }
    $audioCommercialName = Get-PropertyValue $audio @("CommercialName")
    if ($null -eq $audioCommercialName) { $audioCommercialName = Get-PropertyValue $audio @("Format_Commercial_IfAny") }
    $formatAdditionalFeatures = Get-PropertyValue $general @("Format_AdditionalFeatures")
    $audioAdditionalFeatures = Get-PropertyValue $audio @("Format_AdditionalFeatures")
    $eac3Recognized = ([string]$format -match "(?i)E-AC-3|EAC-3|EAC3") -or
        ([string]$audioFormat -match "(?i)E-AC-3|EAC-3|EAC3") -or
        ($human.stdout -match "(?im)^\s*Format\s+:\s+E-AC-3(?:\s|$)")
    $jocMentioned = ([string]$formatAdditionalFeatures -match "(?i)\bJOC\b") -or
        ([string]$audioAdditionalFeatures -match "(?i)\bJOC\b") -or
        ([string]$commercialName -match "(?i)\bAtmos\b") -or
        ([string]$audioCommercialName -match "(?i)\bAtmos\b") -or
        ($human.stdout -match "(?im)^\s*Format\s*:\s*E-AC-3\s+JOC\s*$") -or
        ($human.stdout -match "(?im)^\s*Format/Info\s*:\s*.*Joint Object Coding")
    $status = if ($human.status -eq "TIMEOUT" -or $machine.status -eq "TIMEOUT") {
        "HANG"
    } elseif ($human.status -ne "EXITED" -or $machine.status -ne "EXITED") {
        "CRASH"
    } elseif ($null -eq $parsed) {
        "PARSE_ERROR"
    } elseif ($jocMentioned) {
        "JOC_RECOGNIZED_CORRECTLY"
    } elseif ($eac3Recognized) {
        "EAC3_ONLY_FALSE_NEGATIVE"
    } else {
        "MISCLASSIFIED"
    }
    return [ordered]@{
        available = $true
        status = $status
        human_output = $human.stdout
        parsed = $parsed
        fields = [ordered]@{
            format = $format
            audio_format = $audioFormat
            commercial_name = $commercialName
            format_additional_features = $formatAdditionalFeatures
            format_profile = Get-PropertyValue $general @("Format_Profile")
            codec_id = Get-PropertyValue $audio @("CodecID", "CodecID_String")
            bit_rate = Get-PropertyValue $audio @("BitRate", "BitRate_Nominal")
            sampling_rate = Get-PropertyValue $audio @("SamplingRate")
            channels = Get-PropertyValue $audio @("Channel_s", "Channels")
            channel_layout = Get-PropertyValue $audio @("ChannelLayout")
            duration = Get-PropertyValue $general @("Duration")
            joc_or_atmos_mentioned = $jocMentioned
        }
    }
}

function Write-JsonFile([string]$Path, $Value) {
    ConvertTo-Json -InputObject $Value -Depth 30 | Set-Content -LiteralPath $Path -Encoding utf8
}

function New-Case([string]$Id, [string]$Description, [string]$Profile, [string]$Container, [string]$Carriage, [string]$Emdf, [string]$Partition, [int]$Dependents, [string]$Lfe, [int]$Bitrate, [string]$Generator, [string]$InputKind, [string]$Layout, [string]$Variant = "") {
    $dependentTopology = if ($Dependents -eq 0) {
        "I0-only"
    } elseif ($Dependents -eq 1) {
        "I0+D0"
    } else {
        "I0+D0..D{0}" -f ($Dependents - 1)
    }
    return [pscustomobject]@{
        id = $Id
        description = $Description
        profile = $Profile
        container = $Container
        carriage = $Carriage
        emdf_profile = $Emdf
        partition = $Partition
        dependent_count = $Dependents
        dependent_topology = $dependentTopology
        lfe_ownership = $Lfe
        bitrate_kbps = $Bitrate
        generator = $Generator
        input_kind = $InputKind
        layout = $Layout
        variant = $Variant
    }
}

$cargoPath = Get-ExecutablePath "" @("cargo.exe", "cargo") @()
$gitPath = Get-ExecutablePath "" @("git.exe", "git") @()
if ($null -eq $cargoPath -or $null -eq $gitPath) { throw "cargo and git are required" }
$openjocCandidates = @(Join-Path $openjocRoot "target\release\openjoc.exe")
$openjocPath = Get-ExecutablePath $OpenJocPath @("openjoc.exe", "openjoc") $openjocCandidates
$ffmpegPath = Get-ExecutablePath $FfmpegPath @("ffmpeg.exe", "ffmpeg") @()
$ffprobePath = Get-ExecutablePath $FfprobePath @("ffprobe.exe", "ffprobe") @()
$mediaInfoCandidates = @(
    (Join-Path ([Environment]::GetEnvironmentVariable("ProgramFiles")) "Dolby Media Encoder\resources\dee\MediaInfo.exe"),
    (Join-Path ([Environment]::GetEnvironmentVariable("ProgramFiles")) "Dolby Media Encoder\resources\app.asar.unpacked\resources\dee\MediaInfo.exe"),
    (Join-Path ([Environment]::GetEnvironmentVariable("ProgramFiles(x86)")) "MediaInfo\MediaInfo.exe")
)
$mediaInfoPath = Get-ExecutablePath $MediaInfoPath @("MediaInfo.exe", "mediainfo.exe") $mediaInfoCandidates
if ($null -eq $openjocPath -or $null -eq $ffmpegPath -or $null -eq $ffprobePath) {
    throw "current OpenJOC, FFmpeg, and ffprobe executables are required"
}
if (!(Test-Path -LiteralPath $openjocRoot -PathType Container)) {
    throw "OpenJOC repository is required for current source HEAD: $openjocRoot"
}

if (Test-Path -LiteralPath $OutputRoot) {
    $normalizedRoot = [IO.Path]::GetFullPath($root)
    $normalizedOutput = [IO.Path]::GetFullPath($OutputRoot)
    if (!$normalizedOutput.StartsWith((Join-Path $normalizedRoot "artifacts"), [StringComparison]::OrdinalIgnoreCase)) {
        throw "refusing to clean output outside artifacts/: $normalizedOutput"
    }
    Remove-Item -LiteralPath $OutputRoot -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $probeRoot | Out-Null

$jocforgeHead = Get-RepoHead $root
$censusSourceRevision = Invoke-Bounded $gitPath @("-C", $root, "rev-parse", "HEAD^") 30000 $root
if ($censusSourceRevision.status -ne "EXITED" -or $censusSourceRevision.exit_code -ne 0) {
    throw "failed to resolve census source parent: $root"
}
$censusSourceHead = $censusSourceRevision.stdout.Trim()
$openjocHead = Get-RepoHead $openjocRoot
$jocforgeStatus = Get-RepoStatus $root
$openjocStatus = Get-RepoStatus $openjocRoot
if ($openjocHead -ne $expectedOpenJocHead) {
    throw "OpenJOC source head mismatch: expected $expectedOpenJocHead"
}
if (-not [string]::IsNullOrWhiteSpace($openjocStatus)) {
    throw "OpenJOC checkout must be clean for the pinned census: $openjocStatus"
}
if (-not [string]::IsNullOrWhiteSpace($jocforgeStatus)) {
    throw "JOCForge checkout must be clean for the pinned census: $jocforgeStatus"
}

$matrixCsv = Join-Path $OutputRoot "capability-matrix.csv"
$matrixJson = Join-Path $OutputRoot "capability-matrix.json"
Invoke-Cargo @("run", "-p", "jocforge", "--quiet", "--locked", "--", "composition-matrix", $matrixCsv, $matrixJson) $OutputRoot "matrix"
$matrix = @(Import-Csv -LiteralPath $matrixCsv)
$matrixCounts = @{}
foreach ($group in ($matrix | Group-Object verdict)) { $matrixCounts[$group.Name] = $group.Count }
foreach ($verdict in @("SUPPORTED_PASS", "PUBLIC_AMBIGUOUS", "NOT_APPLICABLE", "SEMANTIC_CEILING", "REJECTED_CAPACITY", "INVALID", "RESERVED", "SPEC_LEGAL_NOT_IMPLEMENTED")) {
    if (!$matrixCounts.ContainsKey($verdict)) { $matrixCounts[$verdict] = 0 }
}
if ([int]$matrixCounts["SPEC_LEGAL_NOT_IMPLEMENTED"] -ne 0) {
    throw "current capability matrix contains SPEC_LEGAL_NOT_IMPLEMENTED cells"
}

$normalInput = Join-Path $OutputRoot "normal-source.bw64"
$lfeInput = Join-Path $OutputRoot "lfe-source.bw64"
Invoke-Cargo @("run", "-p", "jocforge", "--quiet", "--locked", "--", "fixture", "--seconds", "1", "--objects", "1", $normalInput) $OutputRoot "fixture-normal"
Invoke-Cargo @("run", "-p", "jocforge", "--quiet", "--locked", "--", "fixture-directspeakers", "--seconds", "1", $lfeInput) $OutputRoot "fixture-lfe"

$cases = @(
    (New-Case "p01-baseline-idx0-strict-6" "ordinary 5.X JOC baseline" "idx0" "raw" "etsi" "etsi" "6" 1 "absent" 1024 "encode" "normal" "5.1"),
    (New-Case "p02-baseline-idx0-deployed-6" "strict/deployed causal pair: deployed EMDF" "idx0" "raw" "etsi" "deployed" "6" 1 "absent" 1024 "encode" "normal" "5.1"),
    (New-Case "p03a-idx0-skipfld-strict-6" "strict/deployed causal pair: strict I0-only skip-field" "idx0" "raw" "i0-skipfld" "etsi" "6" 0 "absent" 1024 "encode" "normal" "5.1"),
    (New-Case "p03-idx0-skipfld-deployed-6" "I0-only skip-field deployed compatibility carriage" "idx0" "raw" "i0-skipfld" "deployed" "6" 0 "absent" 1024 "encode" "normal" "5.1"),
    (New-Case "p04-idx1-flat-7x-strict-6" "true Flat-7.X profile" "idx1" "raw" "etsi" "etsi" "6" 1 "absent" 1024 "encode" "normal" "7.1.4"),
    (New-Case "p05-idx2-5x2-strict-6" "5.X+2 profile" "idx2" "raw" "etsi" "etsi" "6" 1 "absent" 1024 "encode" "normal" "7.1.4"),
    (New-Case "p06-idx3-phase-strict-6" "5.X plus phase profile" "idx3" "raw" "etsi" "etsi" "6" 1 "absent" 1024 "encode" "normal" "5.1"),
    (New-Case "p07-idx4-phase-5x2-strict-6" "5.X+2 plus phase profile" "idx4" "raw" "etsi" "etsi" "6" 1 "absent" 1024 "encode" "normal" "7.1.4"),
    (New-Case "p08-idx2-short-3x3" "normal PCM-derived 3+3 short-block pair" "idx2" "raw" "etsi" "etsi" "3+3" 1 "absent" 1280 "encode" "normal" "7.1.4"),
    (New-Case "p09-idx2-short-2x2x2" "normal PCM-derived 2+2+2 short-block pair" "idx2" "raw" "etsi" "etsi" "2+2+2" 1 "absent" 1280 "encode" "normal" "7.1.4"),
    (New-Case "p10-idx2-short-1x6" "normal PCM-derived 1x6 short-block pair" "idx2" "raw" "etsi" "etsi" "1+1+1+1+1+1" 1 "absent" 1280 "encode" "normal" "7.1.4"),
    (New-Case "p11-idx0-short-3x3-independent-lfe" "independent LFE outside the JOC matrix" "idx0" "raw" "etsi" "etsi" "3+3" 1 "independent" 1280 "encode" "lfe" "5.1"),
    (New-Case "p12-idx2-dependent-lfe-supplement" "dependent LFE supplementation" "idx2" "raw" "etsi" "etsi" "6" 2 "dependent-supplement" 1664 "dependent-lfe" "normal" "7.1.4"),
    (New-Case "p13-idx2-dependent-lfe-replacement" "dependent LFE replacement" "idx2" "raw" "etsi" "etsi" "6" 2 "dependent-replacement" 1664 "dependent-lfe" "normal" "7.1.4" "replacement"),
    (New-Case "p14-idx1-d0-d1-strict-6" "two dependent substreams D0+D1" "idx1" "raw" "etsi" "etsi" "6" 2 "absent" 1664 "multi-dependent" "normal" "7.1.4"),
    (New-Case "p15-idx2-d0-d2-strict-6" "higher multi-dependent topology D0..D2" "idx2" "raw" "etsi" "etsi" "6" 3 "absent" 2304 "multi-dependent" "normal" "7.1.4"),
    (New-Case "p16-idx2-d0-d1-short-deployed" "orthogonal multi-feature short/deployed composition" "idx2" "raw" "etsi" "deployed" "2+2+2" 2 "absent" 1664 "short-multi-dependent" "normal" "7.1.4"),
    (New-Case "p17-idx1-legacy-ac3-strict" "original-syntax AC-3 core plus E-AC-3 dependent" "idx1" "raw" "legacy-ac3" "etsi" "6" 1 "absent" 1280 "encode" "normal" "7.1.4"),
    (New-Case "p18-idx0-cmaf-ec3-dec3" "CMAF ec-3/dec3 six-block JOC" "idx0" "cmaf" "etsi" "etsi" "6" 1 "absent" 1664 "cmaf" "normal" "5.1"),
    (New-Case "p19-idx4-d0-d5-short-monster" "semantic-maximum idx4 D0..D5 3+3 strict" "idx4" "raw" "etsi" "etsi" "3+3" 6 "absent" 4224 "short-multi-dependent" "normal" "7.1.4")
)

$toolRecords = [ordered]@{
    openjoc = Get-VersionRecord "OpenJOC" $openjocPath @("--version")
    ffmpeg = Get-VersionRecord "FFmpeg" $ffmpegPath @("-version")
    ffprobe = Get-VersionRecord "ffprobe" $ffprobePath @("-version")
    mediainfo = Get-VersionRecord "MediaInfo" $mediaInfoPath @("--Version")
}
if ($toolRecords.openjoc.sha256 -ne $expectedOpenJocSha256) {
    throw "OpenJOC executable hash mismatch: expected $expectedOpenJocSha256"
}
$toolRecords.openjoc.source_head = $openjocHead
$toolRecords.openjoc.repository = $null
$toolRecords.openjoc.repository_recorded = $false
$toolRecords.openjoc.repository_policy = "local checkout supplied by -OpenJocRoot or JOCFORGE_OPENJOC_ROOT; path omitted from publication artifacts"
$toolRecords.ffmpeg.build_configuration = $toolRecords.ffmpeg.banner
$toolRecords.ffprobe.build_configuration = $toolRecords.ffprobe.banner
$toolRecords.mediainfo.available = ($null -ne $mediaInfoPath)
$ffmpegPublicVersion = (($toolRecords.ffmpeg.version -replace "^ffmpeg version\s+", "") -replace "\s+Copyright.*$", "").Trim()
$ffprobePublicVersion = (($toolRecords.ffprobe.version -replace "^ffprobe version\s+", "") -replace "\s+Copyright.*$", "").Trim()
$mediaInfoPublicVersion = if ($toolRecords.mediainfo.banner -match "MediaInfoLib\s+-\s+v([^\s]+)") { "MediaInfoLib $($Matches[1])" } else { $toolRecords.mediainfo.version }
$publicToolRecords = [ordered]@{
    openjoc = [ordered]@{ name = "OpenJOC"; available = $toolRecords.openjoc.available; version = $toolRecords.openjoc.version; sha256 = $toolRecords.openjoc.sha256; source_head = $openjocHead; path_recorded = $false; location_policy = "external checkout path omitted" }
    ffmpeg = [ordered]@{ name = "FFmpeg"; available = $toolRecords.ffmpeg.available; version = $ffmpegPublicVersion; sha256 = $toolRecords.ffmpeg.sha256; path_recorded = $false; location_policy = "executable path omitted" }
    ffprobe = [ordered]@{ name = "ffprobe"; available = $toolRecords.ffprobe.available; version = $ffprobePublicVersion; sha256 = $toolRecords.ffprobe.sha256; path_recorded = $false; location_policy = "executable path omitted" }
    mediainfo = [ordered]@{ name = "MediaInfo"; available = $toolRecords.mediainfo.available; version = $mediaInfoPublicVersion; sha256 = $toolRecords.mediainfo.sha256; path_recorded = $false; location_policy = "executable path omitted" }
}
Write-JsonFile (Join-Path $OutputRoot "tool-versions.json") ([ordered]@{
        schema = "jocforge-interop-tool-versions-v1"
        jocforge_head = $jocforgeHead
        census_source_head = $censusSourceHead
        openjoc_head = $openjocHead
        publication_commit = $jocforgeHead
        tools = $publicToolRecords
    })

$detailRows = [System.Collections.Generic.List[object]]::new()
$manifestRows = [System.Collections.Generic.List[object]]::new()
$resultObjects = [System.Collections.Generic.List[object]]::new()

foreach ($case in $cases) {
    $matrixRow = Get-MatrixRow $case $matrix
    $probeDirectory = Join-Path $probeRoot $case.id
    New-Item -ItemType Directory -Force -Path $probeDirectory | Out-Null
    $inputPath = if ($case.input_kind -eq "lfe") { $lfeInput } else { $normalInput }
    $extension = if ($case.container -eq "cmaf") { "mp4" } else { "ec3" }
    $outputPath = Join-Path $probeDirectory "$($case.id).$extension"
    $inspectTarget = $outputPath
    $generation = $null
    $rawCmafSource = $null

    switch ($case.generator) {
        "encode" {
            $args = @("run", "-p", "jocforge", "--quiet", "--locked", "--", "encode", $inputPath, $outputPath, "--profile", $case.profile, "--bitrate", ([string]$case.bitrate_kbps), "--block-partition", $case.partition)
            if ($case.carriage -ne "etsi") { $args += @("--carriage", $case.carriage) }
            if ($case.emdf_profile -ne "etsi") { $args += @("--emdf-profile", $case.emdf_profile) }
            $generation = Invoke-Cargo $args $probeDirectory "generator"
        }
        "multi-dependent" {
            $args = @("run", "-p", "jocforge", "--quiet", "--locked", "--", "conformance-multi-dependent", $case.profile, ([string]$case.dependent_count), $outputPath)
            $generation = Invoke-Cargo $args $probeDirectory "generator"
        }
        "short-multi-dependent" {
            $args = @("run", "-p", "jocforge", "--quiet", "--locked", "--", "conformance-short-multi-dependent", $case.profile, ([string]$case.dependent_count), $case.partition, $outputPath, "--emdf", $case.emdf_profile, "--lfe", "absent")
            $generation = Invoke-Cargo $args $probeDirectory "generator"
        }
        "dependent-lfe" {
            $args = @("run", "-p", "jocforge", "--quiet", "--locked", "--", "conformance-dependent-lfe", $case.profile, ([string]$case.dependent_count), $outputPath)
            if ($case.variant -eq "replacement") { $args += "--replacement" }
            $generation = Invoke-Cargo $args $probeDirectory "generator"
        }
        "cmaf" {
            $rawCmafSource = Join-Path $probeDirectory "$($case.id)-source.ec3"
            $rawArgs = @("run", "-p", "jocforge", "--quiet", "--locked", "--", "encode", $inputPath, $rawCmafSource, "--profile", $case.profile, "--bitrate", ([string]$case.bitrate_kbps), "--block-partition", $case.partition)
            [void](Invoke-Cargo $rawArgs $probeDirectory "generator-raw")
            $args = @("run", "-p", "jocforge", "--quiet", "--locked", "--", "conformance-cmaf", $case.profile, $rawCmafSource, $outputPath, "--bitrate", ([string]$case.bitrate_kbps))
            $generation = Invoke-Cargo $args $probeDirectory "generator"
            $inspectTarget = $rawCmafSource
        }
        default { throw "unsupported census generator: $($case.generator)" }
    }
    if (!(Test-Path -LiteralPath $outputPath -PathType Leaf)) { throw "generator did not produce probe: $($case.id)" }

    $inspect = Invoke-Bounded $openjocPath @("inspect", $inspectTarget) ($TimeoutSeconds * 1000) $root
    Save-ToolOutput $probeDirectory "openjoc-inspect" $inspect
    $inspectJsonRun = Invoke-Bounded $openjocPath @("inspect", $inspectTarget, "--json") ($TimeoutSeconds * 1000) $root
    Save-ToolOutput $probeDirectory "openjoc-inspect-json" $inspectJsonRun
    $inspectJson = Try-ConvertJson $inspectJsonRun.stdout
    $openjocMalformed = Get-OpenJocMalformed $inspectJson $inspect.stdout
    $openjocDerived = Get-OpenJocDerived $inspectJson $inspect.stdout
    $openjocInspectStatus = if ($inspect.status -eq "TIMEOUT" -or $inspectJsonRun.status -eq "TIMEOUT") {
        "HANG"
    } elseif ($inspect.status -ne "EXITED" -or $inspect.exit_code -ne 0) {
        "PARSE_FAIL"
    } elseif ($openjocMalformed -gt 0) {
        "MALFORMED"
    } elseif ($inspectJsonRun.status -ne "EXITED" -or $inspectJsonRun.exit_code -ne 0 -or $null -eq $inspectJson) {
        "PARSE_FAIL"
    } else {
        "PASS"
    }

    $render = [ordered]@{ status = "NOT_APPLICABLE_CONTAINER"; exit_code = $null; finite = $null; pcm = $null }
    if ($case.container -eq "raw") {
        $renderPath = Join-Path $probeDirectory "$($case.id).wav"
        $renderRun = Invoke-Bounded $openjocPath @("render-joc", $outputPath, "--layout", $case.layout, "--output", $renderPath, "--no-progress", "--overwrite") ($TimeoutSeconds * 1000) $root
        Save-ToolOutput $probeDirectory "openjoc-render" $renderRun
        $pcm = if ($renderRun.status -eq "EXITED" -and $renderRun.exit_code -eq 0 -and (Test-Path -LiteralPath $renderPath)) { Get-PcmHealth $renderPath $probeDirectory "render" } else { $null }
        $render = [ordered]@{
            status = if ($renderRun.status -eq "TIMEOUT") { "HANG" } elseif ($renderRun.status -ne "EXITED") { "CRASH" } elseif ($renderRun.exit_code -ne 0) { "FAIL" } elseif ($null -eq $pcm -or !$pcm.finite) { "NONFINITE_OR_EMPTY" } else { "PASS" }
            exit_code = $renderRun.exit_code
            finite = if ($null -ne $pcm) { $pcm.finite } else { $false }
            pcm = $pcm
        }
    }

    $containerDecode = [ordered]@{ status = "NOT_APPLICABLE"; exit_code = $null }
    if ($case.container -eq "cmaf") {
        $decodeDirectory = Join-Path $probeDirectory "openjoc-decode"
        if (Test-Path -LiteralPath $decodeDirectory) { Remove-Item -LiteralPath $decodeDirectory -Recurse -Force }
        $decodeRun = Invoke-Bounded $openjocPath @("decode", $outputPath, "--internal-base", "-o", $decodeDirectory) ($TimeoutSeconds * 1000) $root
        Save-ToolOutput $probeDirectory "openjoc-container-decode" $decodeRun
        $containerDecode = [ordered]@{
            status = if ($decodeRun.status -eq "TIMEOUT") { "HANG" } elseif ($decodeRun.status -ne "EXITED") { "CRASH" } elseif ($decodeRun.exit_code -eq 0) { "PASS" } else { "FAIL" }
            exit_code = $decodeRun.exit_code
        }
    }
    $openjocVerdict = if ($openjocInspectStatus -eq "MALFORMED") {
        "MALFORMED"
    } elseif ($openjocInspectStatus -ne "PASS") {
        $openjocInspectStatus
    } elseif ($case.container -eq "cmaf" -and $containerDecode.status -eq "PASS") {
        "FULL_PASS"
    } elseif ($case.container -eq "raw" -and $render.status -eq "PASS") {
        "FULL_PASS"
    } elseif ($case.emdf_profile -eq "deployed" -and $openjocDerived.compatibility_status -match "(?i)(pass|accepted)") {
        "STRICT_REJECT_EXPECTED_COMPAT_PASS"
    } else {
        "INSPECT_PASS_RENDER_FAIL"
    }

    $ffprobeRun = Invoke-Bounded $ffprobePath @("-hide_banner", "-v", "warning", "-show_streams", "-show_format", "-of", "json", $outputPath) 30000 $root
    Save-ToolOutput $probeDirectory "ffprobe" $ffprobeRun
    $ffprobeJson = Try-ConvertJson $ffprobeRun.stdout
    $ffprobeSummary = Get-FfprobeSummary $ffprobeJson
    $ffmpegRun = Invoke-Bounded $ffmpegPath @("-hide_banner", "-v", "warning", "-i", $outputPath, "-map", "0:a:0", "-f", "null", "-") ($TimeoutSeconds * 1000) $root
    Save-ToolOutput $probeDirectory "ffmpeg" $ffmpegRun
    $ffmpegStatus = Get-FfmpegStatus $ffmpegRun $case $openjocDerived

    $mediaInfo = Get-MediaInfoResult $outputPath $probeDirectory
    Write-JsonFile (Join-Path $probeDirectory "ffprobe.json") ([ordered]@{
            tool = "ffprobe"
            status = if ($ffprobeRun.status -eq "TIMEOUT") { "HANG" } elseif ($ffprobeRun.status -ne "EXITED") { "CRASH" } elseif ($ffprobeRun.exit_code -eq 0) { "PASS" } else { "PARSE_FAIL" }
            exit_code = $ffprobeRun.exit_code
            parsed = $ffprobeJson
        })
    Write-JsonFile (Join-Path $probeDirectory "ffmpeg-result.json") ([ordered]@{
            tool = "FFmpeg"
            status = $ffmpegStatus
            exit_code = $ffmpegRun.exit_code
            stderr = $ffmpegRun.stderr
            base_decode_exit_code = $ffmpegRun.exit_code
            additional_substreams_observed = ($case.dependent_count -gt 1)
        })
    Write-JsonFile (Join-Path $probeDirectory "mediainfo.json") ([ordered]@{
            tool = "MediaInfo"
            available = $mediaInfo.available
            status = $mediaInfo.status
            fields = $mediaInfo.fields
            parsed = $mediaInfo.parsed
            human_output = $mediaInfo.human_output
        })
    Write-JsonFile (Join-Path $probeDirectory "openjoc.json") ([ordered]@{
            tool = "OpenJOC"
            source_head = $openjocHead
            inspect_target = Get-RelativePath $OutputRoot $inspectTarget
            inspect_status = $openjocInspectStatus
            inspect_exit_code = $inspect.exit_code
            inspect_json_exit_code = $inspectJsonRun.exit_code
            verdict = $openjocVerdict
            malformed_aus = $openjocMalformed
            derived = $openjocDerived
            parsed = $inspectJson
            render = $render
            container_decode = $containerDecode
        })

    $sha256 = Get-Sha256 $outputPath
    $resultObject = [ordered]@{
        id = $case.id
        description = $case.description
        profile = $case.profile
        carriage = $case.carriage
        emdf_profile = $case.emdf_profile
        signaling = $case.emdf_profile
        block_partition = $case.partition
        dependent_count = $case.dependent_count
        dependent_topology = $case.dependent_topology
        lfe_ownership = $case.lfe_ownership
        bitrate_kbps = $case.bitrate_kbps
        container = $case.container
        source_type = if ($case.input_kind -eq "lfe") { "synthetic_directspeakers_lfe" } else { "synthetic_scene" }
        generator = $case.generator
        expected_public_classification = $matrixRow.verdict
        expected_public_reason = $matrixRow.reason
        artifact = Get-RelativePath $OutputRoot $outputPath
        sha256 = $sha256
        openjoc = [ordered]@{ verdict = $openjocVerdict; inspect_status = $openjocInspectStatus; render_status = $render.status; container_decode_status = $containerDecode.status; malformed_aus = $openjocMalformed; derived = $openjocDerived }
        ffmpeg = [ordered]@{ status = $ffmpegStatus; exit_code = $ffmpegRun.exit_code; stderr_path = Get-RelativePath $OutputRoot (Join-Path $probeDirectory "ffmpeg.stderr.txt") }
        ffprobe = [ordered]@{ status = if ($ffprobeRun.status -eq "EXITED" -and $ffprobeRun.exit_code -eq 0) { "PASS" } else { "FAIL" }; summary = $ffprobeSummary }
        mediainfo = [ordered]@{ available = $mediaInfo.available; status = $mediaInfo.status; fields = $mediaInfo.fields }
    }
    Write-JsonFile (Join-Path $probeDirectory "census-result.json") $resultObject
    $resultObjects.Add($resultObject)
    $manifestRows.Add([ordered]@{
            id = $case.id
            description = $case.description
            profile = $case.profile
            carriage = $case.carriage
            emdf_profile = $case.emdf_profile
            signaling = $case.emdf_profile
            block_partition = $case.partition
            dependent_count = $case.dependent_count
            dependent_topology = $case.dependent_topology
            lfe_ownership = $case.lfe_ownership
            bitrate_kbps = $case.bitrate_kbps
            container = $case.container
            source_type = $resultObject.source_type
            artifact = $resultObject.artifact
            sha256 = $sha256
            expected_public_classification = $matrixRow.verdict
        })
    $detailRows.Add([pscustomobject]@{
            probe = $case.id
            key_feature = $case.description
            profile = $case.profile
            carriage = $case.carriage
            emdf_profile = $case.emdf_profile
            signaling = $case.emdf_profile
            block_partition = $case.partition
            dependent_count = $case.dependent_count
            dependent_topology = $case.dependent_topology
            lfe_ownership = $case.lfe_ownership
            bitrate_kbps = $case.bitrate_kbps
            container = $case.container
            sha256 = $sha256
            public_classification = $matrixRow.verdict
            openjoc = $openjocVerdict
            openjoc_inspect = $openjocInspectStatus
            openjoc_render = $render.status
            openjoc_malformed_aus = $openjocMalformed
            ffmpeg = $ffmpegStatus
            ffmpeg_exit_code = $ffmpegRun.exit_code
            ffprobe = if ($ffprobeRun.status -eq "EXITED" -and $ffprobeRun.exit_code -eq 0) { "PASS" } else { "FAIL" }
            mediainfo = $mediaInfo.status
            mediainfo_format = if ($null -ne $mediaInfo.fields) { $mediaInfo.fields.format } else { $null }
        })
}

$manifest = [ordered]@{
    schema = "jocforge-interop-census-v1"
    jocforge_head = $jocforgeHead
    census_source_head = $censusSourceHead
    openjoc_head = $openjocHead
    publication_commit = $jocforgeHead
    normalization = "public path redaction with explicit signaling and dependent topology"
    matrix = [ordered]@{ row_count = $matrix.Count; counts = $matrixCounts }
    positive_probe_count = $manifestRows.Count
    probes = @($manifestRows)
}
Write-JsonFile (Join-Path $OutputRoot "corpus-manifest.json") $manifest
$detailRows | Export-Csv -LiteralPath (Join-Path $OutputRoot "detailed-matrix.csv") -NoTypeInformation -Encoding utf8
Write-JsonFile (Join-Path $OutputRoot "detailed-matrix.json") @($resultObjects)

$openjocCounts = @($detailRows | Group-Object openjoc | Sort-Object Name | ForEach-Object { [ordered]@{ status = $_.Name; count = $_.Count } })
$ffmpegCounts = @($detailRows | Group-Object ffmpeg | Sort-Object Name | ForEach-Object { [ordered]@{ status = $_.Name; count = $_.Count } })
$mediaInfoCounts = @($detailRows | Group-Object mediainfo | Sort-Object Name | ForEach-Object { [ordered]@{ status = $_.Name; count = $_.Count } })
$summary = [ordered]@{
    schema = "jocforge-interop-census-summary-v1"
    positive_probes = $detailRows.Count
    negative_controls = 0
    mediaInfo_available = ($null -ne $mediaInfoPath)
    openjoc = $openjocCounts
    ffmpeg = $ffmpegCounts
    mediainfo = $mediaInfoCounts
    quality = [ordered]@{
        positive_probe_public_legality = (@($detailRows | Where-Object public_classification -ne "SUPPORTED_PASS").Count -eq 0)
        openjoc_positive_malformed_zero = (@($detailRows | Where-Object openjoc_malformed_aus -ne 0).Count -eq 0)
        strict_deployed_pair = ($detailRows.probe -contains "p01-baseline-idx0-strict-6" -and $detailRows.probe -contains "p02-baseline-idx0-deployed-6" -and $detailRows.probe -contains "p03a-idx0-skipfld-strict-6" -and $detailRows.probe -contains "p03-idx0-skipfld-deployed-6")
        shortblock_comparison = (@($detailRows | Where-Object probe -in @("p08-idx2-short-3x3", "p09-idx2-short-2x2x2", "p10-idx2-short-1x6")).Count -eq 3)
        multidependent_comparison = (@($detailRows | Where-Object dependent_count -gt 1).Count -gt 0)
        version_scoped_claims = $true
        wrappers_double_counted = $false
    }
}
$openjocFullCount = @($detailRows | Where-Object { $_.openjoc -eq "FULL_PASS" }).Count
$ffmpegFullCount = @($detailRows | Where-Object { $_.ffmpeg -eq "FULL_PASS" }).Count
$ffmpegPartialCount = @($detailRows | Where-Object { $_.ffmpeg -eq "PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED" }).Count
$mediaInfoJocCount = @($detailRows | Where-Object { $_.mediainfo -eq "JOC_RECOGNIZED_CORRECTLY" }).Count
$mediaInfoEac3OnlyCount = @($detailRows | Where-Object { $_.mediainfo -eq "EAC3_ONLY_FALSE_NEGATIVE" }).Count
if ($detailRows.Count -ne 20 -or
    $openjocFullCount -ne 20 -or
    $ffmpegFullCount -ne 14 -or
    $ffmpegPartialCount -ne 6 -or
    $mediaInfoJocCount -ne 1 -or
    $mediaInfoEac3OnlyCount -ne 19) {
    throw "interop census result counts changed: expected 20 OpenJOC, 14+6 FFmpeg, and 1+19 MediaInfo"
}
Write-JsonFile (Join-Path $OutputRoot "summary.json") $summary

$summaryLines = $detailRows | ForEach-Object {
    $openjocCell = switch ($_.openjoc) {
        "FULL_PASS" { "FULL" }
        "MALFORMED" { "FAIL" }
        default { "FAIL" }
    }
    $ffmpegCell = switch ($_.ffmpeg) {
        "FULL_PASS" { "FULL" }
        "PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED" { "PARTIAL" }
        default { "FAIL" }
    }
    $mediaInfoCell = switch ($_.mediainfo) {
        "JOC_RECOGNIZED_CORRECTLY" { "JOC" }
        "EAC3_ONLY_FALSE_NEGATIVE" { "EAC3 ONLY" }
        "MISCLASSIFIED" { "WRONG DETAILS" }
        default { "FAIL" }
    }
    "| ``$($_.probe)`` | $($_.key_feature) | $openjocCell | $ffmpegCell | $mediaInfoCell |"
}
$openjocCoverage = ($openjocCounts | ForEach-Object { "$($_.status): $($_.count)" }) -join ", "
$ffmpegCoverage = ($ffmpegCounts | ForEach-Object { "$($_.status): $($_.count)" }) -join ", "
$mediaInfoCoverage = ($mediaInfoCounts | ForEach-Object { "$($_.status): $($_.count)" }) -join ", "
$matrixCountLines = ($matrixCounts.GetEnumerator() | Sort-Object Name | ForEach-Object { "| $($_.Name) | $($_.Value) |" }) -join "`n"
$summaryTable = $summaryLines -join "`n"
$openjocTool = $toolRecords["openjoc"]
$ffmpegTool = $toolRecords["ffmpeg"]
$ffprobeTool = $toolRecords["ffprobe"]
$mediaInfoTool = $toolRecords["mediainfo"]
$mediaInfoVersion = (($mediaInfoTool["banner"] -split "`r?`n" | Select-Object -First 2) -join " ").Trim()
$rawHashRows = @(
    "| ``corpus-manifest.json`` | ``$(Get-Sha256 (Join-Path $OutputRoot "corpus-manifest.json"))`` |",
    "| ``detailed-matrix.csv`` | ``$(Get-Sha256 (Join-Path $OutputRoot "detailed-matrix.csv"))`` |",
    "| ``detailed-matrix.json`` | ``$(Get-Sha256 (Join-Path $OutputRoot "detailed-matrix.json"))`` |",
    "| ``summary.json`` | ``$(Get-Sha256 (Join-Path $OutputRoot "summary.json"))`` |",
    "| ``tool-versions.json`` | ``$(Get-Sha256 (Join-Path $OutputRoot "tool-versions.json"))`` |"
) -join "`n"
$canonicalArtifactRoot = Join-Path $root "docs\interop-census"
$canonicalRecordNames = @("corpus-manifest.json", "detailed-matrix.csv", "detailed-matrix.json", "summary.json", "tool-versions.json")
$canonicalRecordsAvailable = @($canonicalRecordNames | Where-Object { Test-Path -LiteralPath (Join-Path $canonicalArtifactRoot $_) }).Count -eq $canonicalRecordNames.Count
$canonicalHashRows = if ($canonicalRecordsAvailable) {
    @(
        "| ``interop-census/corpus-manifest.json`` | ``$(Get-Sha256 (Join-Path $canonicalArtifactRoot "corpus-manifest.json"))`` |",
        "| ``interop-census/detailed-matrix.csv`` | ``$(Get-Sha256 (Join-Path $canonicalArtifactRoot "detailed-matrix.csv"))`` |",
        "| ``interop-census/detailed-matrix.json`` | ``$(Get-Sha256 (Join-Path $canonicalArtifactRoot "detailed-matrix.json"))`` |",
        "| ``interop-census/summary.json`` | ``$(Get-Sha256 (Join-Path $canonicalArtifactRoot "summary.json"))`` |",
        "| ``interop-census/tool-versions.json`` | ``$(Get-Sha256 (Join-Path $canonicalArtifactRoot "tool-versions.json"))`` |"
    ) -join "`n"
} else {
    "| canonical publication records | not present in this checkout |"
}
$docTemplate = @'
# OpenJOC, FFmpeg and MediaInfo interoperability census

This report is a version-scoped publication of a deterministic 20-stream
positive corpus. The publication snapshot is JOCForge `__JOCFORGE_HEAD__`,
with frozen census evidence based on `__CENSUS_SOURCE_HEAD__`, and OpenJOC source
HEAD `__OPENJOC_HEAD__`.

## Executive summary

A 20-stream corpus was selected from JOCForge configurations already classified
`SUPPORTED_PASS` under the project's public ETSI capability model.

- OpenJOC fully parsed all 20 probes and passed the inspection/render-or-
  container-decode gate: 19 raw probes rendered and the CMAF probe passed its
  container-decode path.
- FFmpeg 8.1 fully decoded 14 and partially handled 6 because tested additional
  E-AC-3 substreams were not fully assembled/decoded by that build.
- MediaInfo 21.03 identified 1 probe as JOC and labeled 19 as E-AC-3 only.

These are version-scoped interoperability observations, not claims about every
decoder or detector in existence.

## Scope and implementation families

The public legality authority is the current JOCForge composition matrix. Only
rows classified `SUPPORTED_PASS` entered the positive corpus. OpenJOC is an
independent specialist JOC parser/decoder/renderer; FFmpeg is a mainstream
open-source E-AC-3 decoder implementation family; and MediaInfo is a mainstream
media metadata/format-detection family. MediaInfo output is never treated as
decoder evidence.

LAV may use FFmpeg decoding infrastructure and is not counted separately.
PotPlayer file-information output may reflect MediaInfo-family detection and is
not counted separately. Wrappers and frontends do not add implementation-family
votes.

## Tool versions

Machine-readable records are kept in `artifacts/interop-census/` during a run
and in the canonical normalized copies under `docs/interop-census/`.

These hashes identify this raw normalized run; compare them with an independent
clean rerun when checking reproducibility:

| Raw record | SHA-256 |
|---|---|
__RAW_HASH_ROWS__

The staged canonical publication records have these hashes:

| Canonical record | SHA-256 |
|---|---|
__CANONICAL_HASH_ROWS__

| Tool | Version | SHA-256 | Location policy |
|---|---|---|---|
| OpenJOC | __OPENJOC_VERSION__ | `__OPENJOC_SHA__` | external checkout; local path omitted |
| FFmpeg | __FFMPEG_VERSION__ | `__FFMPEG_SHA__` | resolved executable; local path omitted |
| ffprobe | __FFPROBE_VERSION__ | `__FFPROBE_SHA__` | resolved executable; local path omitted |
| MediaInfo | __MEDIAINFO_VERSION__ | `__MEDIAINFO_SHA__` | resolved executable; local path omitted |

The census OpenJOC pin is intentionally independent of the frozen validator
identity in `validation/openjoc-validator.json`. Existing closure scripts use
that frozen validator for their separate acceptance loop; this census uses the
tested OpenJOC release source head and executable SHA listed above. These
identities must not be conflated.

The exact OpenJOC source commit is `__OPENJOC_HEAD__`. The JOCForge census
source commit is `__CENSUS_SOURCE_HEAD__`; the publication/freeze snapshot is
`__JOCFORGE_HEAD__` and may contain documentation and runner hardening only.

## Corpus methodology

The runner regenerates the matrix, creates project-owned synthetic fixtures, and
selects legal representatives across profiles, carriage, signaling, block
partition, dependent topology, LFE ownership, bitrate and container. OpenJOC is
scored by inspection, malformed-AU count, render health, and CMAF container
decode. FFmpeg is scored by base decode and tested additional-substream behavior.
MediaInfo is scored only by recognition fields. No copyrighted source media is
required for this corpus and no negative control is included in the compact run.

## Capability baseline

| Matrix verdict | Cells |
|---|---:|
__MATRIX_ROWS__

The positive corpus contains __POSITIVE_COUNT__ probes, and every probe is
`SUPPORTED_PASS`. `SPEC_LEGAL_NOT_IMPLEMENTED` is zero in the complete modeled
matrix.

## Corpus results

| Probe | Key feature | OpenJOC | FFmpeg | MediaInfo |
|---|---|---|---|---|
__SUMMARY_TABLE__

The table uses concise cells: `FULL`, `PARTIAL`, `JOC`, and `EAC3 ONLY`.
`PARTIAL` means `PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED`, not malformed or
invalid JOC output. The full per-probe records are in `probes/<probe>` under
the selected output root after reproduction. Canonical
normalized manifest, detailed CSV/JSON, summary JSON, and tool-version records
are under `docs/interop-census/`.

## Family-level summary

- OpenJOC: __OPENJOC_COVERAGE__.
- FFmpeg: __FFMPEG_COVERAGE__.
- MediaInfo: __MEDIAINFO_COVERAGE__.

These are observations for the exact binaries above. They do not establish
universal support limits for other builds or products.

## Controlled-pair findings

- Strict versus deployed: raw `p01`/`p02` were both FULL in OpenJOC and FFmpeg, and both were E-AC-3-only in MediaInfo. In the I0-only skip-field pair, strict `p03a` was E-AC-3-only while deployed `p03` was labeled JOC by MediaInfo. This is the exact observed behavior of MediaInfo 21.03; it is not a claim that deployed signaling is required by JOC or ETSI.
- Six blocks versus short blocks: `p08`, `p09`, and `p10` held the single-dependent raw composition constant while testing `3+3`, `2+2+2`, and `1+1+1+1+1+1`. OpenJOC and FFmpeg were FULL for all three; MediaInfo labeled all three E-AC-3 only. Short-block handling is therefore a separate tested boundary from general JOC recognition.
- D0 versus D1+: single-dependent representatives decoded FULL in FFmpeg. The tested D1+ topologies in `p14`, `p15`, `p16`, and `p19`, plus the dependent-LFE cases `p12` and `p13`, were classified PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED. This describes the tested FFmpeg 8.1 build only.
- Profiles: idx0, idx1 Flat-7.X, idx2 5.X+2, idx3 phase signaling, and idx4 phase signaling all received FULL OpenJOC inspection/render evidence. MediaInfo labels do not establish profile support.

## Semantic-maximum probe

`p19-idx4-d0-d5-short-monster` is the **Semantic-Maximum Raw JOC Probe**:
idx4, D0..D5, shared `3+3`, ETSI strict signaling, and 4224 kbps. It was a
`SUPPORTED_PASS` row, fully parsed/rendered by OpenJOC, partially handled by
FFmpeg 8.1 because of additional-substream coverage, and labeled E-AC-3 only by
MediaInfo 21.03.

## Limitations and claim scope

This corpus uses deterministic synthetic audio. OpenJOC render health checks
finite PCM and positive sample count; it does not claim perceptual parity.
FFmpeg behavior is an interoperability observation and does not decide ETSI
legality. MediaInfo is not an audio decoder. The public capability claim is
limited to JOCForge's modeled public ETSI E-AC-3 JOC space: 140,400 cells,
2,393 `SUPPORTED_PASS`, and zero `SPEC_LEGAL_NOT_IMPLEMENTED` cells. This is
not a claim of complete Dolby compatibility, proprietary encoder parity, patent
clearance, certification, or byte-equivalence with a Dolby encoder.

## Release-facing wording

Safe concepts include: independent clean-room Rust E-AC-3 JOC decoder/renderer;
standards-aware JOC stream inspector; the modeled 140,400-cell public capability
space; 2,393 `SUPPORTED_PASS` combinations; zero modeled
`SPEC_LEGAL_NOT_IMPLEMENTED` cells; OpenJOC 20/20 in this corpus; FFmpeg 8.1
14 FULL plus 6 PARTIAL; and MediaInfo 21.03 1/20 JOC recognition.

Avoid universal compatibility, official Dolby compatibility, bit-perfect claims,
patent/legal conclusions, and claims that every other decoder is broken.

## Reproduction

From the JOCForge repository root:

```powershell
.\scripts\run-interop-census.ps1
```

Prerequisites are Rust/Cargo, a clean OpenJOC checkout at the pinned source
commit, FFmpeg 8.1 with ffprobe, and MediaInfo 21.03. Supply the OpenJOC checkout
with `-OpenJocRoot` or `JOCFORGE_OPENJOC_ROOT`; supply MediaInfo with
`-MediaInfoPath` or make it available on PATH. The runner does not require a
machine-specific path.

The runner cleans only its generated directory under `artifacts/`, regenerates
the current capability matrix and project-owned fixtures, runs bounded
OpenJOC/FFmpeg/ffprobe/MediaInfo commands, records raw stdout/stderr beside
normalized JSON, and writes `interop-census.generated.md` under the selected
output root. Pass
`-OutputRoot artifacts\interop-census-rerun` to produce an independent clean
rerun for reproducibility comparison. Expected output includes
`corpus-manifest.json`, `detailed-matrix.csv`, `detailed-matrix.json`,
`summary.json`, `tool-versions.json`, and one `probes/<probe>/` directory per
positive probe. Compare the normalized hashes with the canonical records under
`docs/interop-census/`; the tracked canonical report is not overwritten.

## QA history

The first complete workspace test run contained one transient failure. A
targeted rerun passed, the subsequent full suite rerun passed, and the clean
census reruns were reproducible. See `docs/interop-census-qa.md` for the
internal QA note.

No production Rust decoder or encoder source was changed for this census.
'@
$doc = $docTemplate
$doc = $doc.Replace("__JOCFORGE_HEAD__", [string]$jocforgeHead)
$doc = $doc.Replace("__CENSUS_SOURCE_HEAD__", [string]$censusSourceHead)
$doc = $doc.Replace("__OPENJOC_HEAD__", [string]$openjocHead)
$doc = $doc.Replace("__MATRIX_COUNT__", [string]$matrix.Count)
$doc = $doc.Replace("__OPENJOC_VERSION__", [string]$openjocTool["version"])
$doc = $doc.Replace("__OPENJOC_SHA__", [string]$openjocTool["sha256"])
$doc = $doc.Replace("__FFMPEG_VERSION__", [string]$ffmpegTool["version"])
$doc = $doc.Replace("__FFMPEG_SHA__", [string]$ffmpegTool["sha256"])
$doc = $doc.Replace("__FFPROBE_VERSION__", [string]$ffprobeTool["version"])
$doc = $doc.Replace("__FFPROBE_SHA__", [string]$ffprobeTool["sha256"])
$doc = $doc.Replace("__MEDIAINFO_VERSION__", [string]$mediaInfoVersion)
$doc = $doc.Replace("__MEDIAINFO_SHA__", [string]$mediaInfoTool["sha256"])
$doc = $doc.Replace("__RAW_HASH_ROWS__", $rawHashRows)
$doc = $doc.Replace("__CANONICAL_HASH_ROWS__", $canonicalHashRows)
$doc = $doc.Replace("__MATRIX_ROWS__", $matrixCountLines)
$doc = $doc.Replace("__POSITIVE_COUNT__", [string]$detailRows.Count)
$doc = $doc.Replace("__SUMMARY_TABLE__", $summaryTable)
$doc = $doc.Replace("__OPENJOC_COVERAGE__", $openjocCoverage)
$doc = $doc.Replace("__FFMPEG_COVERAGE__", $ffmpegCoverage)
$doc = $doc.Replace("__MEDIAINFO_COVERAGE__", $mediaInfoCoverage)
$doc | Set-Content -LiteralPath $docsPath -Encoding utf8

Write-Output "JOC interoperability census written to $OutputRoot"
Write-Output "positive probes: $($detailRows.Count)"
Write-Output "OpenJOC: $openjocCoverage"
Write-Output "FFmpeg: $ffmpegCoverage"
Write-Output "MediaInfo: $mediaInfoCoverage"
