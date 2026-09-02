$ErrorActionPreference = "Stop"

function Get-OpenJocSha256([string]$Path) {
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

function Assert-OpenJocValidatorPin([string]$RepositoryRoot, [string]$OpenJocPath) {
    $pinPath = Join-Path $RepositoryRoot "validation\openjoc-validator.json"
    if (!(Test-Path -LiteralPath $pinPath)) {
        throw "OpenJOC validator pin does not exist: $pinPath"
    }
    if (!(Test-Path -LiteralPath $OpenJocPath)) {
        throw "OpenJOC executable does not exist: $OpenJocPath"
    }
    $pin = Get-Content -LiteralPath $pinPath -Raw | ConvertFrom-Json
    $actualHash = Get-OpenJocSha256 $OpenJocPath
    if ($actualHash -ne $pin.executable_sha256) {
        throw "OpenJOC validator SHA256 does not match pin: expected '$($pin.executable_sha256)', got '$actualHash'"
    }
    $versionOutput = (& $OpenJocPath --version 2>&1) -join "`n"
    $expectedVersion = "OpenJOC $($pin.semantic_version)"
    if ($versionOutput.Trim() -ne $expectedVersion) {
        throw "OpenJOC validator version does not match pin: expected '$expectedVersion', got '$($versionOutput.Trim())'"
    }
    [pscustomobject]@{
        tool = $pin.tool
        semantic_version = $pin.semantic_version
        source_revision = $pin.source_revision
        source_description = $pin.source_description
        executable_sha256 = $actualHash
        executable = [IO.Path]::GetFullPath($OpenJocPath)
        version_output = $versionOutput.Trim()
        platform = $pin.platform
    }
}
