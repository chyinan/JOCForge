# pattern: Imperative Shell

[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$failures = [System.Collections.Generic.List[string]]::new()

$workflowPath = Join-Path $root ".github\workflows\release-windows.yml"
$configurePath = Join-Path $root "release\ffmpeg-configure.sh"
$pinPath = Join-Path $root "release\ffmpeg-pin.json"
$winpthreadsLicensePath = Join-Path $root "release\winpthreads-COPYING.txt"

if (!(Test-Path -LiteralPath $workflowPath -PathType Leaf)) {
    $failures.Add("release workflow is missing: $workflowPath")
}
if (!(Test-Path -LiteralPath $configurePath -PathType Leaf)) {
    $failures.Add("authoritative FFmpeg configure script is missing: $configurePath")
}
if (!(Test-Path -LiteralPath $winpthreadsLicensePath -PathType Leaf)) {
    $failures.Add("static winpthreads license evidence is missing: $winpthreadsLicensePath")
}

if (Test-Path -LiteralPath $workflowPath -PathType Leaf) {
    $workflow = Get-Content -LiteralPath $workflowPath -Raw
    $workflowConfigureMatch = [regex]::Match($workflow, '(?m)^\s*bash\s+([^\s]+ffmpeg-configure\.sh)\s+')
    if (!$workflowConfigureMatch.Success) {
        $failures.Add("release workflow does not invoke the authoritative FFmpeg configure script")
    }
    else {
        $workflowConfigure = $workflowConfigureMatch.Groups[1].Value.Replace('/', '\\')
        $resolved = Join-Path $root $workflowConfigure
        if (!(Test-Path -LiteralPath $resolved -PathType Leaf)) {
            $failures.Add("release workflow references a missing file: $workflowConfigure")
        }
    }

    $referencedFiles = [regex]::Matches($workflow, '(?m)(?<![A-Za-z0-9_.-])((?:release|scripts)/[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*\.(?:sh|ps1))') |
        ForEach-Object { $_.Groups[1].Value } |
        Sort-Object -Unique
    foreach ($relative in $referencedFiles) {
        $resolved = Join-Path $root ($relative.Replace('/', '\\'))
        if (!(Test-Path -LiteralPath $resolved -PathType Leaf)) {
            $failures.Add("release workflow references a missing repository file: $relative")
        }
    }
}

$configure = if (Test-Path -LiteralPath $configurePath -PathType Leaf) {
    Get-Content -LiteralPath $configurePath -Raw
}
else {
    ""
}
foreach ($required in @(
    "--extra-ldflags=-static",
    "--enable-demuxer=pcm_f32le",
    "--enable-filter=aresample"
)) {
    if ($configure -notmatch [regex]::Escape($required)) {
        $failures.Add("authoritative configure script is missing required component: $required")
    }
}
if ($configure -match [regex]::Escape("--enable-demuxer=f32le")) {
    $failures.Add("authoritative configure script still uses the invalid f32le demuxer name")
}
if ($configure -notmatch "canonical_prefix=/jocforge-ffmpeg-prefix" -or
    $configure.Contains('--prefix="$prefix"')) {
    $failures.Add("FFmpeg configure would embed the machine-specific installation prefix in binaries")
}

if (Test-Path -LiteralPath $pinPath -PathType Leaf) {
    $pin = Get-Content -LiteralPath $pinPath -Raw | ConvertFrom-Json
    $pinArguments = @($pin.ffmpeg.build_configuration)
    foreach ($required in @(
        "--extra-ldflags=-static",
        "--enable-demuxer=pcm_f32le",
        "--enable-filter=aresample"
    )) {
        if ($pinArguments -notcontains $required) {
            $failures.Add("release pin is missing required configure argument: $required")
        }
    }
    if ($pinArguments -contains "--enable-demuxer=f32le") {
        $failures.Add("release pin still uses the invalid f32le demuxer name")
    }
}
else {
    $failures.Add("FFmpeg release pin is missing: $pinPath")
}

$assemblyPath = Join-Path $root "scripts\release\assemble-windows-release.ps1"
$smokePath = Join-Path $root "scripts\release\smoke-windows-bundle.ps1"
foreach ($requiredPath in @($assemblyPath, $smokePath)) {
    if (!(Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
        $failures.Add("release script is missing: $requiredPath")
    }
}
if (Test-Path -LiteralPath $assemblyPath -PathType Leaf) {
    $assembly = Get-Content -LiteralPath $assemblyPath -Raw
    if ($assembly -notmatch "Normalize-FfmpegBuildConfiguration") {
        $failures.Add("assembly does not use normalized FFmpeg build configuration evidence")
    }
    if ($assembly -notmatch "--prefix=<INSTALL_PREFIX>" -or
        $assembly -notmatch "WINPTHREADS-COPYING.txt" -or
        $assembly -notmatch "LIBWINPTHREAD_DLL_DISTRIBUTED=NO") {
        $failures.Add("assembly does not publish stable static-runtime provenance")
    }
    if ($assembly -match "Compress-Archive") {
        $failures.Add("assembly still uses non-deterministic Compress-Archive")
    }
}
if (Test-Path -LiteralPath $smokePath -PathType Leaf) {
    $smoke = Get-Content -LiteralPath $smokePath -Raw
    if ($smoke -notmatch "private.*path|private_content|libwinpthread") {
        $failures.Add("release smoke does not contain private-path/runtime dependency checks")
    }
    if ($smoke -notmatch "UNSUPPORTED_DECODER_FEATURE") {
        $failures.Add("release smoke does not classify the known FFmpeg additional-substream limitation")
    }
    $backslash = [char]92
    $privatePathLiterals = @(
        ("D:" + $backslash + "Programs"),
        ("D:/" + "Programs"),
        ("D:" + $backslash + "ADM BWF"),
        ("D:/" + "ADM BWF"),
        ("C:" + $backslash + "Users"),
        ("C:/" + "Users")
    )
    foreach ($privatePathLiteral in $privatePathLiterals) {
        if ($smoke.Contains($privatePathLiteral)) {
            $failures.Add("release smoke contains a machine-specific private path literal")
            break
        }
    }
}

if ($failures.Count -gt 0) {
    Write-Output "release contract test FAIL"
    $failures | ForEach-Object { Write-Output (" - " + $_) }
    exit 1
}
Write-Output "release contract test PASS"
