# pattern: Imperative Shell

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$FfmpegPrefix,
    [Parameter(Mandatory = $true)]
    [string]$FfmpegSourceRoot,
    [Parameter(Mandatory = $true)]
    [string]$FfmpegSourceArchive,
    [Parameter(Mandatory = $true)]
    [string]$JocforgeExecutable,
    [Parameter(Mandatory = $true)]
    [string]$OutputRoot,
    [string]$Version = "0.1.0-dev",
    [switch]$CiBuilt
)

$ErrorActionPreference = "Stop"
$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$pinPath = Join-Path $repositoryRoot "release\ffmpeg-pin.json"
$pin = Get-Content -LiteralPath $pinPath -Raw | ConvertFrom-Json

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

function Require-File([string]$Path, [string]$Description) {
    if (!(Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Description does not exist: $Path"
    }
}

function Invoke-CapturedProcess([string]$File, [string[]]$Arguments) {
    $captureRoot = Join-Path ([IO.Path]::GetTempPath()) "jocforge-release-capture-$PID"
    if (Test-Path -LiteralPath $captureRoot) {
        throw "release capture directory already exists: $captureRoot"
    }
    New-Item -ItemType Directory -Path $captureRoot | Out-Null
    $stdoutPath = Join-Path $captureRoot "stdout.txt"
    $stderrPath = Join-Path $captureRoot "stderr.txt"
    try {
        $process = Start-Process -FilePath $File -ArgumentList $Arguments -NoNewWindow -Wait -PassThru `
            -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
        [pscustomobject]@{
            exit_code = $process.ExitCode
            stdout = if (Test-Path -LiteralPath $stdoutPath) { [IO.File]::ReadAllText($stdoutPath) } else { "" }
            stderr = if (Test-Path -LiteralPath $stderrPath) { [IO.File]::ReadAllText($stderrPath) } else { "" }
        }
    }
    finally {
        if (Test-Path -LiteralPath $captureRoot) {
            Remove-Item -LiteralPath $captureRoot -Recurse -Force
        }
    }
}

function Normalize-FfmpegBuildConfiguration([string[]]$Lines, $Pin) {
    [string[]]$bestTokens = @()
    for ($index = 0; $index -lt $Lines.Count; $index++) {
        $match = [regex]::Match($Lines[$index], '^\s*configuration:\s*(.*)$')
        if (!$match.Success) {
            continue
        }
        $tokens = [System.Collections.Generic.List[string]]::new()
        $inline = $match.Groups[1].Value.Trim()
        if ($inline.Length -gt 0) {
            foreach ($token in ($inline -split '\s+')) {
                $tokens.Add([string]$token)
            }
        }
        for ($next = $index + 1; $next -lt $Lines.Count; $next++) {
            $trimmed = $Lines[$next].Trim()
            if ([string]::IsNullOrWhiteSpace($trimmed) -or $trimmed -like 'Exiting with exit code*') {
                break
            }
            if ($trimmed.StartsWith('--')) {
                foreach ($token in ($trimmed -split '\s+')) {
                    $tokens.Add([string]$token)
                }
            }
            elseif ($tokens.Count -gt 0) {
                break
            }
        }
        if ($tokens.Count -gt 0) {
            if ($tokens.Count -gt $bestTokens.Count) {
                $bestTokens = [string[]]$tokens.ToArray()
            }
        }
    }
    if ($bestTokens.Count -eq 0) {
        throw "FFmpeg -buildconf did not expose a configuration argument list"
    }
    $actual = @($bestTokens | ForEach-Object {
        if ($_ -match '^--prefix=') {
            '--prefix=<INSTALL_PREFIX>'
        }
        else {
            [string]$_
        }
    })
    $expected = @('--prefix=<INSTALL_PREFIX>') + @($Pin.ffmpeg.build_configuration | Where-Object { $_ -ne './configure' })
    if (($actual -join "`n") -ne ($expected -join "`n")) {
        throw "effective FFmpeg configuration differs from the pinned canonical configuration"
    }
    if ($actual -contains '--enable-gpl' -or
        $actual -contains '--enable-nonfree' -or
        $actual -contains '--enable-version3') {
        throw "FFmpeg build configuration enables a forbidden license class"
    }
    [pscustomobject]@{
        tokens = $actual
        text = $actual -join [Environment]::NewLine
    }
}

function Write-DeterministicZip([string]$SourceRoot, [string]$DestinationPath) {
    Add-Type -AssemblyName System.IO.Compression
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $files = @(Get-ChildItem -LiteralPath $SourceRoot -File -Recurse | ForEach-Object {
        [pscustomobject]@{
            full_name = $_.FullName
            relative = $_.FullName.Substring($SourceRoot.Length + 1).Replace('\', '/')
        }
    } | Sort-Object relative)
    $stream = [IO.File]::Open($DestinationPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    $archive = [IO.Compression.ZipArchive]::new($stream, [IO.Compression.ZipArchiveMode]::Create, $false)
    try {
        $timestamp = [DateTimeOffset]::new(2000, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
        foreach ($file in $files) {
            $entry = $archive.CreateEntry($file.relative, [IO.Compression.CompressionLevel]::Optimal)
            $entry.LastWriteTime = $timestamp
            $input = [IO.File]::OpenRead($file.full_name)
            $output = $entry.Open()
            try {
                $input.CopyTo($output)
            }
            finally {
                $output.Dispose()
                $input.Dispose()
            }
        }
    }
    finally {
        $archive.Dispose()
        $stream.Dispose()
    }
}

Require-File $JocforgeExecutable "JOCForge executable"
Require-File (Join-Path $FfmpegPrefix "bin\ffmpeg.exe") "FFmpeg executable"
Require-File (Join-Path $FfmpegPrefix "bin\ffprobe.exe") "ffprobe executable"
Require-File $FfmpegSourceArchive "FFmpeg source archive"
Require-File (Join-Path $FfmpegSourceRoot "LICENSE.md") "FFmpeg license"
Require-File (Join-Path $FfmpegSourceRoot "COPYING.LGPLv2.1") "FFmpeg LGPL license"
$winpthreadsLicenseRelative = [string]$pin.ffmpeg.winpthreads.license_file
if ([string]::IsNullOrWhiteSpace($winpthreadsLicenseRelative)) {
    throw "FFmpeg release pin is missing the static winpthreads license file"
}
Require-File (Join-Path $repositoryRoot ($winpthreadsLicenseRelative.Replace('/', '\\'))) "winpthreads license"

if ((Get-Sha256 $FfmpegSourceArchive) -ne [string]$pin.ffmpeg.source_sha256) {
    throw "FFmpeg source archive SHA256 does not match the release pin"
}
$ffmpegPath = Join-Path $FfmpegPrefix "bin\ffmpeg.exe"
$ffprobePath = Join-Path $FfmpegPrefix "bin\ffprobe.exe"
$buildConfigurationRun = Invoke-CapturedProcess $ffmpegPath @('-buildconf')
if ($buildConfigurationRun.exit_code -ne 0) {
    throw "FFmpeg build configuration query failed"
}
$buildConfigurationLines = @(
    @($buildConfigurationRun.stdout -split "`r?`n")
    @($buildConfigurationRun.stderr -split "`r?`n")
)
$normalizedConfiguration = Normalize-FfmpegBuildConfiguration $buildConfigurationLines $pin

if ($Version -notmatch '^[A-Za-z0-9._-]+$') {
    throw "release version contains unsafe path characters: $Version"
}

$bundleName = "JOCForge-v$Version-win-x64"
New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
$bundleRoot = Join-Path $OutputRoot $bundleName
if (Test-Path -LiteralPath $bundleRoot) {
    throw "release bundle directory already exists: $bundleRoot"
}
New-Item -ItemType Directory -Path $bundleRoot | Out-Null
$ffmpegLicenseRoot = Join-Path $bundleRoot "licenses\FFmpeg"
New-Item -ItemType Directory -Force -Path $ffmpegLicenseRoot | Out-Null

Copy-Item -LiteralPath $JocforgeExecutable -Destination (Join-Path $bundleRoot "jocforge.exe")
Copy-Item -LiteralPath (Join-Path $FfmpegPrefix "bin\ffmpeg.exe") -Destination (Join-Path $bundleRoot "ffmpeg.exe")
Copy-Item -LiteralPath (Join-Path $FfmpegPrefix "bin\ffprobe.exe") -Destination (Join-Path $bundleRoot "ffprobe.exe")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "LICENSE") -Destination (Join-Path $bundleRoot "LICENSE")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "THIRD_PARTY_NOTICES.md") -Destination (Join-Path $bundleRoot "THIRD_PARTY_NOTICES.md")
Copy-Item -LiteralPath (Join-Path $FfmpegSourceRoot "LICENSE.md") -Destination (Join-Path $ffmpegLicenseRoot "LICENSE.md")
Copy-Item -LiteralPath (Join-Path $FfmpegSourceRoot "COPYING.LGPLv2.1") -Destination (Join-Path $ffmpegLicenseRoot "COPYING.LGPLv2.1")
$sourceArchiveName = [IO.Path]::GetFileName($FfmpegSourceArchive)
$sourceArchiveInBundle = Join-Path $ffmpegLicenseRoot $sourceArchiveName
Copy-Item -LiteralPath $FfmpegSourceArchive -Destination $sourceArchiveInBundle
Copy-Item -LiteralPath (Join-Path $repositoryRoot ($winpthreadsLicenseRelative.Replace('/', '\\'))) `
    -Destination (Join-Path $ffmpegLicenseRoot "WINPTHREADS-COPYING.txt")

$sourceInfo = @(
    "FFMPEG_VERSION=$($pin.ffmpeg.version)",
    "FFMPEG_TAG=$($pin.ffmpeg.tag)",
    "FFMPEG_COMMIT=$($pin.ffmpeg.commit)",
    "FFMPEG_SOURCE_URL=$($pin.ffmpeg.source_url)",
    "FFMPEG_SOURCE_SHA256=$($pin.ffmpeg.source_sha256)",
    "LOCAL_FFMPEG_MODIFICATIONS=NONE",
    "WINPTHREADS_LINKAGE=$($pin.ffmpeg.winpthreads.linkage.ToUpperInvariant())",
    "WINPTHREADS_LICENSE=$($pin.ffmpeg.winpthreads.license)",
    "WINPTHREADS_LICENSE_FILE=licenses/FFmpeg/WINPTHREADS-COPYING.txt",
    "LIBWINPTHREAD_DLL_DISTRIBUTED=NO",
    "SOURCE_ARCHIVE_INCLUDED=YES",
    "SOURCE_ARCHIVE_FILENAME=$([IO.Path]::GetFileName($FfmpegSourceArchive))"
)
$sourceInfo | Set-Content -LiteralPath (Join-Path $ffmpegLicenseRoot "SOURCE_INFO.txt") -Encoding utf8
@(
    "JOCFORGE_FFMPEG_BUILD_CONFIG_FORMAT=1",
    "FFMPEG_VERSION=$($pin.ffmpeg.version)",
    "FFMPEG_LICENSE=$($pin.ffmpeg.license)",
    "FFMPEG_GPL=DISABLED",
    "FFMPEG_NONFREE=DISABLED",
    "FFMPEG_VERSION3=DISABLED",
    "WINPTHREADS_LINKAGE=STATIC",
    "WINPTHREADS_LICENSE=$($pin.ffmpeg.winpthreads.license)",
    "",
    "Canonical configure arguments:",
    "--prefix=<INSTALL_PREFIX>",
    ($pin.ffmpeg.build_configuration | Where-Object { $_ -ne "./configure" }),
    "",
    "Effective ffmpeg -buildconf arguments (normalized):",
    $normalizedConfiguration.tokens
) | Set-Content -LiteralPath (Join-Path $ffmpegLicenseRoot "BUILD_CONFIG.txt") -Encoding utf8
"No local FFmpeg source modifications were applied. The build used the pinned upstream source archive directly." | Set-Content -LiteralPath (Join-Path $ffmpegLicenseRoot "NO_LOCAL_CHANGES.txt") -Encoding utf8

$bundleFfmpegPath = Join-Path $bundleRoot "ffmpeg.exe"
$bundleFfprobePath = Join-Path $bundleRoot "ffprobe.exe"
$manifest = [ordered]@{
    schema_version = 1
    bundle = $bundleName
    platform = "windows-x86_64"
    jocforge = [ordered]@{
        executable = "jocforge.exe"
        license = "Apache-2.0"
        sha256 = Get-Sha256 (Join-Path $bundleRoot "jocforge.exe")
    }
    ffmpeg = [ordered]@{
        version = $pin.ffmpeg.version
        tag = $pin.ffmpeg.tag
        commit = $pin.ffmpeg.commit
        source_url = $pin.ffmpeg.source_url
        source_sha256 = $pin.ffmpeg.source_sha256
        license = $pin.ffmpeg.license
        gpl_enabled = [bool]$pin.ffmpeg.gpl_enabled
        nonfree_enabled = [bool]$pin.ffmpeg.nonfree_enabled
        version3_enabled = [bool]$pin.ffmpeg.version3_enabled
        local_modifications = "NONE"
        ci_built = $CiBuilt.IsPresent
        executable = "ffmpeg.exe"
        executable_sha256 = Get-Sha256 $bundleFfmpegPath
        build_configuration = @($normalizedConfiguration.tokens)
        winpthreads = $pin.ffmpeg.winpthreads
    }
    ffprobe = [ordered]@{
        executable = "ffprobe.exe"
        executable_sha256 = Get-Sha256 $bundleFfprobePath
    }
    files = [ordered]@{
        jocforge_bytes = (Get-Item -LiteralPath (Join-Path $bundleRoot "jocforge.exe")).Length
        ffmpeg_bytes = (Get-Item -LiteralPath $bundleFfmpegPath).Length
        ffprobe_bytes = (Get-Item -LiteralPath $bundleFfprobePath).Length
    }
    archive_normalization = [ordered]@{
        format = "zip"
        file_order = "lexicographic-relative-path"
        entry_timestamp_utc = "2000-01-01T00:00:00Z"
        compression = "System.IO.Compression.Optimal"
    }
    openjoc_included = $false
    system_ffmpeg_required = $false
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $bundleRoot "manifest.json") -Encoding utf8

$hashFiles = @(Get-ChildItem -LiteralPath $bundleRoot -File -Recurse | ForEach-Object {
    [pscustomobject]@{
        full_name = $_.FullName
        relative = $_.FullName.Substring($bundleRoot.Length + 1).Replace('\', '/')
    }
} | Where-Object { $_.relative -ne "SHA256SUMS.txt" } | Sort-Object relative)
$hashLines = foreach ($file in $hashFiles) {
    "$(Get-Sha256 $file.full_name)  $($file.relative)"
}
$hashLines | Set-Content -LiteralPath (Join-Path $bundleRoot "SHA256SUMS.txt") -Encoding ascii

$zipPath = Join-Path $OutputRoot "$bundleName.zip"
if (Test-Path -LiteralPath $zipPath) {
    throw "release ZIP already exists: $zipPath"
}
Write-DeterministicZip $bundleRoot $zipPath
$externalChecksums = @("$(Get-Sha256 $zipPath)  $bundleName.zip") + @(
    foreach ($file in @(Get-ChildItem -LiteralPath $bundleRoot -File -Recurse | ForEach-Object {
        [pscustomobject]@{
            full_name = $_.FullName
            relative = $_.FullName.Substring($bundleRoot.Length + 1).Replace('\', '/')
        }
    } | Sort-Object relative)) {
        "$(Get-Sha256 $file.full_name)  $bundleName/$($file.relative)"
    }
)
$externalChecksums | Set-Content -LiteralPath (Join-Path $OutputRoot "SHA256SUMS.txt") -Encoding ascii
$artifact = [ordered]@{
    bundle = $bundleName
    zip = [IO.Path]::GetFileName($zipPath)
    zip_sha256 = Get-Sha256 $zipPath
    zip_size_bytes = (Get-Item -LiteralPath $zipPath).Length
    extracted_size_bytes = ((Get-ChildItem -LiteralPath $bundleRoot -File -Recurse | Measure-Object -Property Length -Sum).Sum)
    jocforge_sha256 = $manifest.jocforge.sha256
    jocforge_bytes = $manifest.files.jocforge_bytes
    ffmpeg_bytes = $manifest.files.ffmpeg_bytes
    ffprobe_bytes = $manifest.files.ffprobe_bytes
    ffmpeg_sha256 = $manifest.ffmpeg.executable_sha256
    ffprobe_sha256 = $manifest.ffprobe.executable_sha256
    ffmpeg_version = $pin.ffmpeg.version
    ffmpeg_commit = $pin.ffmpeg.commit
    ffmpeg_source_sha256 = $pin.ffmpeg.source_sha256
    effective_ffmpeg_license = $pin.ffmpeg.license
    ffmpeg_gpl_enabled = $false
    ffmpeg_nonfree_enabled = $false
    ffmpeg_ci_built = $CiBuilt.IsPresent
    ffmpeg_build_configuration_sha256 = Get-Sha256 (Join-Path $ffmpegLicenseRoot "BUILD_CONFIG.txt")
    binary_distribution_license_gate = "REVIEW_REQUIRED_UNTIL_SMOKE"
    archive_normalization = $manifest.archive_normalization
}
$artifact | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $OutputRoot "release-artifact.json") -Encoding utf8
Write-Output (Join-Path $OutputRoot "release-artifact.json")
