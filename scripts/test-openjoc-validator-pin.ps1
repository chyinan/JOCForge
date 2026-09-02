param(
    [string]$OpenJoc = "D:\Programs\OpenJOC\target\debug\openjoc.exe"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot "openjoc-validator-pin.ps1")

$identity = Assert-OpenJocValidatorPin $root $OpenJoc
$fake = Join-Path ([IO.Path]::GetTempPath()) "jocforge-wrong-openjoc-$PID.exe"
try {
    [IO.File]::WriteAllBytes($fake, [byte[]](0..31))
    $rejected = $false
    try {
        Assert-OpenJocValidatorPin $root $fake | Out-Null
    }
    catch {
        $rejected = $true
    }
    if (!$rejected) {
        throw "wrong OpenJOC validator was not rejected"
    }
    [ordered]@{
        pass = $true
        pinned_version = $identity.semantic_version
        pinned_sha256 = $identity.executable_sha256
        wrong_validator_rejected = $rejected
    } | ConvertTo-Json -Depth 4
}
finally {
    if (Test-Path -LiteralPath $fake) {
        Remove-Item -LiteralPath $fake -Force
    }
}
