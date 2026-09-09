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

Require-File $JocforgeExecutable "JOCForge executable"
Require-File (Join-Path $FfmpegPrefix "bin\ffmpeg.exe") "FFmpeg executable"
Require-File (Join-Path $FfmpegPrefix "bin\ffprobe.exe") "ffprobe executable"
Require-File $FfmpegSourceArchive "FFmpeg source archive"
Require-File (Join-Path $FfmpegSourceRoot "LICENSE.md") "FFmpeg license"
Require-File (Join-Path $FfmpegSourceRoot "COPYING.LGPLv2.1") "FFmpeg LGPL license"

if ((Get-Sha256 $FfmpegSourceArchive) -ne [string]$pin.ffmpeg.source_sha256) {
    throw "FFmpeg source archive SHA256 does not match the release pin"
}
$buildConfiguration = ((& (Join-Path $FfmpegPrefix "bin\ffmpeg.exe") -buildconf 2>&1) -join "`n")
if ($LASTEXITCODE -ne 0) {
    throw "FFmpeg build configuration query failed"
}
if ($buildConfiguration -match "--enable-gpl|--enable-nonfree|--enable-version3") {
    throw "FFmpeg binary has a GPL, nonfree, or version-3 configuration"
}

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

$sourceInfo = @(
    "FFMPEG_VERSION=$($pin.ffmpeg.version)",
    "FFMPEG_TAG=$($pin.ffmpeg.tag)",
    "FFMPEG_COMMIT=$($pin.ffmpeg.commit)",
    "FFMPEG_SOURCE_URL=$($pin.ffmpeg.source_url)",
    "FFMPEG_SOURCE_SHA256=$($pin.ffmpeg.source_sha256)",
    "LOCAL_FFMPEG_MODIFICATIONS=NONE",
    "SOURCE_ARCHIVE_INCLUDED=YES",
    "SOURCE_ARCHIVE_FILENAME=$([IO.Path]::GetFileName($FfmpegSourceArchive))"
)
$sourceInfo | Set-Content -LiteralPath (Join-Path $ffmpegLicenseRoot "SOURCE_INFO.txt") -Encoding utf8
@(
    "Requested configure arguments:",
    ($pin.ffmpeg.build_configuration -join [Environment]::NewLine),
    "",
    "Actual ffmpeg -buildconf output:",
    $buildConfiguration
) | Set-Content -LiteralPath (Join-Path $ffmpegLicenseRoot "BUILD_CONFIG.txt") -Encoding utf8
"No local FFmpeg source modifications were applied. The build used the pinned upstream source archive directly." | Set-Content -LiteralPath (Join-Path $ffmpegLicenseRoot "NO_LOCAL_CHANGES.txt") -Encoding utf8

$ffmpegPath = Join-Path $bundleRoot "ffmpeg.exe"
$ffprobePath = Join-Path $bundleRoot "ffprobe.exe"
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
        executable_sha256 = Get-Sha256 $ffmpegPath
    }
    ffprobe = [ordered]@{
        executable = "ffprobe.exe"
        executable_sha256 = Get-Sha256 $ffprobePath
    }
    files = [ordered]@{
        jocforge_bytes = (Get-Item -LiteralPath (Join-Path $bundleRoot "jocforge.exe")).Length
        ffmpeg_bytes = (Get-Item -LiteralPath $ffmpegPath).Length
        ffprobe_bytes = (Get-Item -LiteralPath $ffprobePath).Length
    }
    openjoc_included = $false
    system_ffmpeg_required = $false
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $bundleRoot "manifest.json") -Encoding utf8

$hashLines = foreach ($file in Get-ChildItem -LiteralPath $bundleRoot -File -Recurse | Sort-Object FullName) {
    if ($file.Name -ne "SHA256SUMS.txt") {
        $relative = $file.FullName.Substring($bundleRoot.Length + 1).Replace('\', '/')
        "$(Get-Sha256 $file.FullName)  $relative"
    }
}
$hashLines | Set-Content -LiteralPath (Join-Path $bundleRoot "SHA256SUMS.txt") -Encoding ascii

$zipPath = Join-Path $OutputRoot "$bundleName.zip"
Compress-Archive -Path (Join-Path $bundleRoot "*") -DestinationPath $zipPath -CompressionLevel Optimal
$externalChecksums = @(
    "$(Get-Sha256 $zipPath)  $bundleName.zip",
    "$(Get-Sha256 (Join-Path $bundleRoot "jocforge.exe"))  $bundleName/jocforge.exe",
    "$(Get-Sha256 $ffmpegPath)  $bundleName/ffmpeg.exe",
    "$(Get-Sha256 $ffprobePath)  $bundleName/ffprobe.exe",
    "$(Get-Sha256 $sourceArchiveInBundle)  $bundleName/licenses/FFmpeg/$sourceArchiveName"
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
    binary_distribution_license_gate = "REVIEW_REQUIRED_UNTIL_SMOKE"
}
$artifact | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $OutputRoot "release-artifact.json") -Encoding utf8
Write-Output (Join-Path $OutputRoot "release-artifact.json")
