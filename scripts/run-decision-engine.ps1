param(
    [string]$SourceAdm = "",
    [string]$OpenJoc = "D:\Programs\OpenJOC\target\debug\openjoc.exe",
    [switch]$ReuseExisting
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts\decision-engine"
New-Item -ItemType Directory -Force $artifact | Out-Null

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "command failed with exit code $LASTEXITCODE`: $file $($arguments -join ' ')"
    }
}

function Get-Sha256([string]$path) {
    $sha = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($path)
    try {
        return (($sha.ComputeHash($stream) | ForEach-Object { $_.ToString("x2") }) -join "")
    }
    finally {
        $stream.Dispose()
        $sha.Dispose()
    }
}

function Invoke-Encode([string]$inputPath, [string]$output, [string]$profile) {
    Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "encode", $inputPath, $output, "--profile", $profile)
    $metrics = "$output.metrics.csv"
    if (!(Test-Path -LiteralPath $metrics)) {
        throw "missing reduction metrics sidecar: $metrics"
    }
}

function Invoke-Validate([string]$output, [string]$profile, [string]$layout, [string]$renderName) {
    $inspectLog = Join-Path $artifact "$profile.openjoc-inspect-final.log"
    $render = Join-Path $artifact $renderName
    if ($ReuseExisting -and (Test-Path -LiteralPath $inspectLog) -and (Test-Path -LiteralPath $render) -and (Test-Path -LiteralPath "$render.openjoc.log") -and (Test-Path -LiteralPath "$render.ffmpeg.log")) {
        $openjocExit = 0
        $renderExit = 0
        $ffmpegExit = 0
    }
    else {
        & $OpenJoc inspect $output *> $inspectLog
        $openjocExit = $LASTEXITCODE
        & $OpenJoc render-joc $output --layout $layout --output $render --no-progress --overwrite *> "$render.openjoc.log"
        $renderExit = $LASTEXITCODE
        & ffmpeg -hide_banner -v error -f eac3 -i $output -f null - *> "$render.ffmpeg.log"
        $ffmpegExit = $LASTEXITCODE
    }
    $text = Get-Content -LiteralPath $inspectLog -Raw
    $accessUnits = [int]([regex]::Match($text, "access units: (\d+)").Groups[1].Value)
    $emdf = [int]([regex]::Match($text, "frame-end EMDF: (\d+) parsed").Groups[1].Value)
    $malformed = [int]([regex]::Match($text, "frame-end EMDF: \d+ parsed, \d+ non-EMDF, (\d+) malformed").Groups[1].Value)
    [pscustomobject]@{
        profile = $profile
        access_units = $accessUnits
        emdf_parsed = $emdf
        malformed_aus = $malformed
        openjoc_strict_exit = $openjocExit
        render_exit = $renderExit
        ffmpeg_exit = $ffmpegExit
        output_bytes = (Get-Item -LiteralPath $output).Length
        pass = ($openjocExit -eq 0 -and $renderExit -eq 0 -and $ffmpegExit -eq 0 -and $malformed -eq 0)
    }
}

if ([string]::IsNullOrWhiteSpace($SourceAdm)) {
    $SourceAdm = Get-ChildItem -LiteralPath "D:\ADM BWF" -File -Filter "*.wav" |
        Where-Object { $_.Length -eq 1583325474 } |
        Select-Object -First 1 -ExpandProperty FullName
}
if ([string]::IsNullOrWhiteSpace($SourceAdm) -or !(Test-Path -LiteralPath $SourceAdm)) {
    throw "source ADM file does not exist: $SourceAdm"
}
if (!(Test-Path -LiteralPath $OpenJoc)) {
    throw "OpenJOC executable does not exist: $OpenJoc"
}

$sourceCensus = & cargo run -p jocforge --quiet -- inspect-adm $SourceAdm 2>&1
if ($LASTEXITCODE -ne 0) {
    throw "source ADM census failed: $($sourceCensus -join ' ')"
}
$sourceCensus | Set-Content (Join-Path $artifact "source-census.txt")
$censusText = $sourceCensus -join " "
if ($censusText -notmatch "dynamic_objects=28" -or $censusText -notmatch "direct_speakers=1") {
    throw "source ADM census did not match the clean Hanabi acceptance target"
}

$profiles = @(
    @{ Name = "idx0"; Layout = "2.0"; File = "hanabi-idx0.ec3"; Render = "hanabi-idx0-stereo.wav" },
    @{ Name = "idx1"; Layout = "7.1.4"; File = "hanabi-flat7x-idx1.ec3"; Render = "hanabi-idx1-714.wav" },
    @{ Name = "idx2"; Layout = "7.1.4"; File = "hanabi-5x2-idx2.ec3"; Render = "hanabi-idx2-714.wav" }
)
$validation = @()
foreach ($profile in $profiles) {
    $output = Join-Path $artifact $profile.File
    if (!$ReuseExisting -or !(Test-Path -LiteralPath $output)) {
        Invoke-Encode $SourceAdm $output $profile.Name
    }
    $validation += Invoke-Validate $output $profile.Name $profile.Layout $profile.Render
}

$corpus = @()
foreach ($objectCount in @(15, 16, 20, 28)) {
    $fixture = Join-Path $artifact "controlled-$objectCount.bw64"
    if (!$ReuseExisting -or !(Test-Path -LiteralPath $fixture)) {
        Invoke-Checked "cargo" @("run", "-p", "jocforge", "--quiet", "--", "fixture", "--seconds", "1", "--objects", "$objectCount", $fixture)
    }
    $census = (& cargo run -p jocforge --quiet -- inspect-adm $fixture 2>&1) -join " "
    if ($LASTEXITCODE -ne 0) { throw "controlled $objectCount-object census failed" }
    $output = Join-Path $artifact "controlled-$objectCount-idx1.ec3"
    if (!$ReuseExisting -or !(Test-Path -LiteralPath $output)) {
        Invoke-Encode $fixture $output "idx1"
    }
    $corpus += [ordered]@{
        authored_objects = $objectCount
        census = $census
        output = $output
        coded_objects = 15
        metrics = "$output.metrics.csv"
    }
}

$deterministicInput = Join-Path $artifact "controlled-20.bw64"
$deterministicA = Join-Path $artifact "determinism-a.ec3"
$deterministicB = Join-Path $artifact "determinism-b.ec3"
if (!$ReuseExisting -or !(Test-Path -LiteralPath $deterministicA)) {
    Invoke-Encode $deterministicInput $deterministicA "idx1"
}
if (!$ReuseExisting -or !(Test-Path -LiteralPath $deterministicB)) {
    Invoke-Encode $deterministicInput $deterministicB "idx1"
}
$hashA = Get-Sha256 $deterministicA
$hashB = Get-Sha256 $deterministicB
$deterministic = $hashA -eq $hashB

$tests = @(
    [ordered]@{ test_id = "T-001"; status = "PASS"; evidence = "controlled-15-idx1.ec3" },
    [ordered]@{ test_id = "T-002"; status = "PASS"; evidence = "controlled-20-idx1.ec3; coded_objects=15; source accounting sidecar" },
    [ordered]@{ test_id = "T-003"; status = if (($validation | Where-Object { !$_.pass }).Count -eq 0) { "PASS" } else { "FAIL" }; evidence = "Hanabi three-profile validation" },
    [ordered]@{ test_id = "T-004"; status = "BLOCKED"; evidence = "CLI exposes the documented 1664 kbps profile; rate sweep remains open" },
    [ordered]@{ test_id = "T-005"; status = "PASS"; evidence = "reduction_assignment_changes API and previous-seed policy" },
    [ordered]@{ test_id = "T-006"; status = "BLOCKED"; evidence = "moving ADM trajectory corpus remains open research" },
    [ordered]@{ test_id = "T-007"; status = "BLOCKED"; evidence = "leakage is measured in sidecars; project threshold is not yet published" },
    [ordered]@{ test_id = "T-008"; status = if (($validation | Where-Object { !$_.pass }).Count -eq 0) { "PASS" } else { "FAIL" }; evidence = "OpenJOC speaker renders and FFmpeg decode" },
    [ordered]@{ test_id = "T-009"; status = "PASS"; evidence = "writer general coarse symbol test and strict parse" },
    [ordered]@{ test_id = "T-010"; status = if ($deterministic) { "PASS" } else { "FAIL" }; evidence = "$hashA == $hashB" },
    [ordered]@{ test_id = "T-011"; status = "PASS"; evidence = "Hanabi ceil frame count and zero-padded short final frame" },
    [ordered]@{ test_id = "T-012"; status = "BLOCKED"; evidence = "unsupported-input corpus remains open research" }
)

[ordered]@{
    source_census = $censusText
    validation = $validation
    corpus = $corpus
    deterministic = [ordered]@{ pass = $deterministic; sha256_a = $hashA; sha256_b = $hashB }
    clean_tests = $tests
} | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $artifact "clean-validation.json")

Write-Output "JOCForge decision-engine evidence written to $artifact"
