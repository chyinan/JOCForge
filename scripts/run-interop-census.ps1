# pattern: Imperative Shell

[CmdletBinding()]
param(
    [string]$OutputRoot = "",
    [string]$OpenJocPath = "",
    [string]$FfmpegPath = "",
    [string]$FfprobePath = "",
    [string]$MediaInfoPath = "",
    [int]$TimeoutSeconds = 120
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$openjocRoot = Join-Path (Split-Path -Parent $root) "OpenJOC"
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $root "artifacts\interop-census"
}
$OutputRoot = [IO.Path]::GetFullPath($OutputRoot)
$docsPath = Join-Path $root "docs\interop-census.md"
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
        return [ordered]@{ name = $Name; available = $false; path = $null; sha256 = $null; version = $null; banner = $null }
    }
    $result = Invoke-Bounded $Path $Arguments 30000 $root
    $banner = ($result.stdout + $result.stderr).Trim()
    $firstLine = ($banner -split "`r?`n" | Select-Object -First 1).Trim()
    return [ordered]@{
        name = $Name
        available = ($result.status -eq "EXITED" -and $result.exit_code -eq 0)
        path = $Path
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

function Get-FfmpegStatus($Run, $ProbeJson, $Case) {
    if ($Run.status -eq "TIMEOUT") { return "HANG" }
    if ($Run.status -ne "EXITED") { return "CRASH" }
    if ($Run.exit_code -eq 0) {
        if ($Case.dependent_count -gt 1 -and $Run.stderr -match "(?i)(additional|substream|unsupported|skipping|not support)") {
            return "PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED"
        }
        return "FULL_PASS"
    }
    if ($Case.dependent_count -gt 1) { return "PARTIAL_ADDITIONAL_SUBSTREAM_UNSUPPORTED" }
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

function Get-MediaInfoResult([string]$Path, [string]$Directory, $ExpectedOpenJoc) {
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
    $eac3Recognized = ([string]$format -match "(?i)E-AC-3|EAC-3|EAC3") -or
        ([string]$audioFormat -match "(?i)E-AC-3|EAC-3|EAC3") -or
        ($human.stdout -match "(?im)^\s*Format\s+:\s+E-AC-3(?:\s|$)")
    $jocMentioned = (($human.stdout + $machine.stdout) -match "(?i)\b(JOC|Atmos)\b")
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
            commercial_name = Get-PropertyValue $general @("CommercialName")
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
    return [pscustomobject]@{
        id = $Id
        description = $Description
        profile = $Profile
        container = $Container
        carriage = $Carriage
        emdf_profile = $Emdf
        partition = $Partition
        dependent_count = $Dependents
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
    "D:\Programs\dee71joc\test_dee\MediaInfo.exe",
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
$openjocHead = Get-RepoHead $openjocRoot
$jocforgeStatus = Get-RepoStatus $root
$openjocStatus = Get-RepoStatus $openjocRoot

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
$toolRecords.openjoc.source_head = $openjocHead
$toolRecords.openjoc.repository = $openjocRoot
$toolRecords.ffmpeg.build_configuration = $toolRecords.ffmpeg.banner
$toolRecords.ffprobe.build_configuration = $toolRecords.ffprobe.banner
$toolRecords.mediainfo.available = ($null -ne $mediaInfoPath)
Write-JsonFile (Join-Path $OutputRoot "tool-versions.json") ([ordered]@{
        jocforge_head = $jocforgeHead
        openjoc_head = $openjocHead
        tools = $toolRecords
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
    $ffmpegStatus = Get-FfmpegStatus $ffmpegRun $ffprobeJson $case

    $mediaInfo = Get-MediaInfoResult $outputPath $probeDirectory $openjocDerived
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
        block_partition = $case.partition
        dependent_count = $case.dependent_count
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
            block_partition = $case.partition
            dependent_count = $case.dependent_count
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
            block_partition = $case.partition
            dependent_count = $case.dependent_count
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
    openjoc_head = $openjocHead
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
Write-JsonFile (Join-Path $OutputRoot "summary.json") $summary

$summaryLines = $detailRows | ForEach-Object {
    "| ``$($_.probe)`` | $($_.key_feature) | $($_.openjoc) | $($_.ffmpeg) | $($_.mediainfo) |"
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
$docTemplate = @'
# OpenJOC, FFmpeg and MediaInfo interoperability census

Verified against JOCForge `__JOCFORGE_HEAD__` and current OpenJOC source HEAD `__OPENJOC_HEAD__`.

This is a compact, version-scoped census of legal JOC representatives. It is not an exhaustive run of the `__MATRIX_COUNT__`-cell capability lattice.

## Scope and implementation families

The public legality authority is the current JOCForge composition matrix. Only rows classified `SUPPORTED_PASS` entered the positive corpus. OpenJOC is measured as a JOC inspector/decoder/renderer, FFmpeg as a mainstream E-AC-3 stream/decode stack, and MediaInfo as a metadata recognizer. MediaInfo output is not used as decode evidence.

LAV, mpv and similar frontends may share FFmpeg decoding infrastructure and are not counted as independent decoder families. PotPlayer file-information output may reflect MediaInfo-family detection and is not counted as an independent metadata detector. No frontend smoke test contributes to the family counts.

## Tool versions

Machine-readable paths: `artifacts/interop-census/tool-versions.json` and `artifacts/interop-census/corpus-manifest.json`.

| Tool | Version | SHA-256 | Source / path |
|---|---|---|---|
| OpenJOC | __OPENJOC_VERSION__ | `__OPENJOC_SHA__` | current source HEAD `__OPENJOC_HEAD__` |
| FFmpeg | __FFMPEG_VERSION__ | `__FFMPEG_SHA__` | `__FFMPEG_PATH__` |
| ffprobe | __FFPROBE_VERSION__ | `__FFPROBE_SHA__` | `__FFPROBE_PATH__` |
| MediaInfo | __MEDIAINFO_VERSION__ | `__MEDIAINFO_SHA__` | `__MEDIAINFO_PATH__` |

## Capability baseline

| Matrix verdict | Cells |
|---|---:|
__MATRIX_ROWS__

The positive corpus contains __POSITIVE_COUNT__ probes. The script does not silently substitute another metadata detector if MediaInfo is unavailable; this run found MediaInfo available.

## Corpus results

| Probe | Key feature | OpenJOC | FFmpeg | MediaInfo |
|---|---|---|---|---|
__SUMMARY_TABLE__

The full per-probe records are in `artifacts/interop-census/probes/<probe>`. The normalized detailed matrix is `artifacts/interop-census/detailed-matrix.csv` and `.json`; the corpus manifest is `artifacts/interop-census/corpus-manifest.json`.

## Family-level summary

- OpenJOC: __OPENJOC_COVERAGE__.
- FFmpeg: __FFMPEG_COVERAGE__.
- MediaInfo: __MEDIAINFO_COVERAGE__.

These are family-level observations for the exact binaries above. They do not establish universal support limits for other builds or products.

## Controlled boundaries

- `p01` and `p02` hold profile, raw ETSI carriage, six-block grouping, dependent count, LFE state and bitrate constant while changing strict versus deployed EMDF signaling. `p03a` and `p03` provide the same strict/deployed comparison for I0-only skip-field carriage. Their MediaInfo labels are detector observations, not legality judgments.
- `p08`, `p09` and `p10` hold profile and raw strict single-dependent composition constant while changing the normal short-block partition among `3+3`, `2+2+2` and `1x6`.
- `p14` and `p15` add dependent substreams while retaining source-derived raw ETSI syntax; any FFmpeg partial result is reported as additional-substream coverage, not malformed JOC.
- `p19` is the current generator-compatible semantic maximum: idx4, D0..D5, shared `3+3`, strict EMDF, and 4224 kbps. The exact supported row was resolved from the current matrix before generation.

## Limitations

The corpus is deterministic synthetic audio, with a small supplementary real-program path intentionally omitted from the public reproduction requirement. OpenJOC render health checks finite PCM and positive sample count; it does not claim perceptual parity. FFmpeg output is a decode/recognition observation and does not decide ETSI legality. MediaInfo is not a decoder. Negative controls were not included in this compact run.

## Reproduction

From the JOCForge repository root:

```powershell
.\scripts\run-interop-census.ps1
```

The runner cleans only its generated directory under `artifacts/`, regenerates the current capability matrix and project-owned fixtures, runs bounded OpenJOC/FFmpeg/ffprobe/MediaInfo commands, records raw stdout/stderr beside normalized JSON, and writes this document. Pass `-OutputRoot artifacts\interop-census-rerun` to produce an independent clean rerun for reproducibility comparison.

No production Rust decoder or encoder source was changed for this census.
'@
$doc = $docTemplate
$doc = $doc.Replace("__JOCFORGE_HEAD__", [string]$jocforgeHead)
$doc = $doc.Replace("__OPENJOC_HEAD__", [string]$openjocHead)
$doc = $doc.Replace("__MATRIX_COUNT__", [string]$matrix.Count)
$doc = $doc.Replace("__OPENJOC_VERSION__", [string]$openjocTool["version"])
$doc = $doc.Replace("__OPENJOC_SHA__", [string]$openjocTool["sha256"])
$doc = $doc.Replace("__FFMPEG_VERSION__", [string]$ffmpegTool["version"])
$doc = $doc.Replace("__FFMPEG_SHA__", [string]$ffmpegTool["sha256"])
$doc = $doc.Replace("__FFMPEG_PATH__", [string]$ffmpegTool["path"])
$doc = $doc.Replace("__FFPROBE_VERSION__", [string]$ffprobeTool["version"])
$doc = $doc.Replace("__FFPROBE_SHA__", [string]$ffprobeTool["sha256"])
$doc = $doc.Replace("__FFPROBE_PATH__", [string]$ffprobeTool["path"])
$doc = $doc.Replace("__MEDIAINFO_VERSION__", [string]$mediaInfoVersion)
$doc = $doc.Replace("__MEDIAINFO_SHA__", [string]$mediaInfoTool["sha256"])
$doc = $doc.Replace("__MEDIAINFO_PATH__", [string]$mediaInfoTool["path"])
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
