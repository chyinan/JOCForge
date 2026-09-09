# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")
$OpenJoc = (Resolve-OpenJocValidator $root).executable

$identity = Assert-OpenJocValidatorPin $root $OpenJoc
$frozenHashBefore = Get-OpenJocSha256 $OpenJoc
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) "jocforge-openjoc-pin-tests-$PID"
$fake = Join-Path $tempRoot "openjoc.exe"
$activeOpenJocTarget = Join-Path $tempRoot "active-openjoc.exe"
$wrongVersionRoot = Join-Path $tempRoot "wrong-version"
try {
    New-Item -ItemType Directory -Force (Join-Path $wrongVersionRoot "validation") | Out-Null
    [IO.File]::WriteAllBytes($fake, [byte[]](0..31))
    [IO.File]::WriteAllBytes($activeOpenJocTarget, [byte[]](0..31))
    $wrongHashRejected = $false
    try {
        Assert-OpenJocValidatorPin $root $fake | Out-Null
    }
    catch {
        $wrongHashRejected = $true
    }
    if (!$wrongHashRejected) {
        throw "wrong OpenJOC hash was not rejected"
    }
    $activeTargetRejected = $false
    try {
        Assert-OpenJocValidatorPin $root $activeOpenJocTarget | Out-Null
    }
    catch {
        $activeTargetRejected = $true
    }
    if (!$activeTargetRejected) {
        throw "active OpenJOC target was accepted as the authoritative validator"
    }
    $pin = Get-Content -LiteralPath (Join-Path $root "validation\openjoc-validator.json") -Raw
    $pin.Replace('"semantic_version": "0.16.0"', '"semantic_version": "9.99.9"') |
        Set-Content -LiteralPath (Join-Path $wrongVersionRoot "validation\openjoc-validator.json") -Encoding utf8
    $wrongVersionRejected = $false
    try {
        Assert-OpenJocValidatorPin $wrongVersionRoot $OpenJoc | Out-Null
    }
    catch {
        $wrongVersionRejected = $true
    }
    if (!$wrongVersionRejected) {
        throw "wrong OpenJOC version was not rejected"
    }
    $frozenHashAfter = Get-OpenJocSha256 $OpenJoc
    $activeOpenJocRebuildIndependence = $activeTargetRejected -and
        $frozenHashBefore -eq $frozenHashAfter -and
        [IO.Path]::GetFullPath($activeOpenJocTarget) -ne [IO.Path]::GetFullPath($identity.executable)
    [ordered]@{
        pass = $wrongHashRejected -and $wrongVersionRejected -and $activeOpenJocRebuildIndependence
        pinned_version = $identity.semantic_version
        pinned_sha256 = $identity.executable_sha256
        pinned_source_revision = $identity.source_revision
        wrong_hash_rejected = $wrongHashRejected
        wrong_version_rejected = $wrongVersionRejected
        path_substitution_rejected = $wrongHashRejected
        active_openjoc_target_rejected = $activeTargetRejected
        active_openjoc_rebuild_independence_test = $activeOpenJocRebuildIndependence
    } | ConvertTo-Json -Depth 4
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}
