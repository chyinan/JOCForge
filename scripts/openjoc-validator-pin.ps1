# pattern: Imperative Shell

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

function Get-OpenJocValidatorPin([string]$RepositoryRoot) {
    $pinPath = Join-Path $RepositoryRoot "validation\openjoc-validator.json"
    if (!(Test-Path -LiteralPath $pinPath)) {
        throw "OpenJOC validator pin does not exist: $pinPath"
    }
    $pin = Get-Content -LiteralPath $pinPath -Raw | ConvertFrom-Json
    foreach ($property in @("tool", "semantic_version", "source_revision", "executable_sha256", "artifact_manifest", "frozen_executable")) {
        if ([string]::IsNullOrWhiteSpace([string]$pin.$property)) {
            throw "OpenJOC validator pin is missing '$property': $pinPath"
        }
    }
    if ($pin.tool -ne "OpenJOC" -or
        $pin.source_revision -notmatch "^[0-9a-f]{40}$" -or
        $pin.executable_sha256 -notmatch "^[0-9a-f]{64}$") {
        throw "OpenJOC validator pin is malformed: $pinPath"
    }
    $pin
}

function Assert-OpenJocValidatorPin([string]$RepositoryRoot, [string]$OpenJocPath = "") {
    $pin = Get-OpenJocValidatorPin $RepositoryRoot
    $manifestPath = [IO.Path]::GetFullPath($pin.artifact_manifest)
    $frozenPath = [IO.Path]::GetFullPath($pin.frozen_executable)
    if (!(Test-Path -LiteralPath $manifestPath) -or !(Test-Path -LiteralPath $frozenPath)) {
        throw "frozen OpenJOC validator artifact is incomplete: $manifestPath"
    }
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    if ($manifest.purpose -ne "JOCForge authoritative external validator" -or
        $manifest.immutable_artifact -ne $true -or
        $manifest.tool -ne $pin.tool -or
        $manifest.version -ne $pin.semantic_version -or
        $manifest.source_commit -ne $pin.source_revision -or
        $manifest.executable_sha256 -ne $pin.executable_sha256 -or
        [IO.Path]::GetFullPath($manifest.executable) -ne $frozenPath) {
        throw "frozen OpenJOC validator manifest does not match pin: $manifestPath"
    }
    if ((Get-Item -LiteralPath $frozenPath).IsReadOnly -ne $true) {
        throw "frozen OpenJOC validator executable is not read-only: $frozenPath"
    }
    if ($frozenPath -match "[\\/]Programs[\\/]OpenJOC[\\/]target[\\/]") {
        throw "active OpenJOC target cannot be an authoritative validator: $frozenPath"
    }
    if (![string]::IsNullOrWhiteSpace($OpenJocPath) -and
        [IO.Path]::GetFullPath($OpenJocPath) -ne $frozenPath) {
        throw "OpenJOC validator path is not the frozen artifact: $OpenJocPath"
    }
    $actualHash = Get-OpenJocSha256 $frozenPath
    if ($actualHash -ne $pin.executable_sha256) {
        throw "frozen OpenJOC validator SHA256 does not match pin: expected '$($pin.executable_sha256)', got '$actualHash'"
    }
    $versionOutput = (& $frozenPath --version 2>&1) -join "`n"
    $expectedVersion = "OpenJOC $($pin.semantic_version)"
    if ($versionOutput.Trim() -ne $expectedVersion) {
        throw "frozen OpenJOC validator version does not match pin: expected '$expectedVersion', got '$($versionOutput.Trim())'"
    }
    [pscustomobject]@{
        tool = $pin.tool
        semantic_version = $pin.semantic_version
        source_revision = $pin.source_revision
        source_description = $manifest.source_description
        executable_sha256 = $actualHash
        executable = $frozenPath
        artifact_manifest = $manifestPath
        cargo_lock_sha256 = $manifest.cargo_lock_sha256
        rustc = $manifest.rustc
        cargo = $manifest.cargo
        target = $manifest.target
        build_profile = $manifest.build_profile
        version_output = $versionOutput.Trim()
        platform = $pin.platform
        old_pin_sha256 = $pin.old_pin_sha256
    }
}

function Resolve-OpenJocValidator([string]$RepositoryRoot) {
    Assert-OpenJocValidatorPin $RepositoryRoot
}
