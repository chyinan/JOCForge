param(
    [string]$OpenJoc = "D:\Programs\OpenJOC\target\debug\openjoc.exe"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")

$identity = Assert-OpenJocValidatorPin $root $OpenJoc
$tempRoot = Join-Path ([IO.Path]::GetTempPath()) "jocforge-openjoc-pin-tests-$PID"
$fake = Join-Path $tempRoot "openjoc.exe"
$wrongVersionRoot = Join-Path $tempRoot "wrong-version"
try {
    New-Item -ItemType Directory -Force (Join-Path $wrongVersionRoot "validation") | Out-Null
    [IO.File]::WriteAllBytes($fake, [byte[]](0..31))
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
    [ordered]@{
        pass = $wrongHashRejected -and $wrongVersionRejected
        pinned_version = $identity.semantic_version
        pinned_sha256 = $identity.executable_sha256
        pinned_source_revision = $identity.source_revision
        wrong_hash_rejected = $wrongHashRejected
        wrong_version_rejected = $wrongVersionRejected
        path_substitution_rejected = $wrongHashRejected
    } | ConvertTo-Json -Depth 4
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}
