# pattern: Imperative Shell

param(
    [string]$SourceAdm = ""
)

$ErrorActionPreference = "Stop"
$script = Join-Path $PSScriptRoot "run-decision-engine.ps1"
& $script -SourceAdm $SourceAdm
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
