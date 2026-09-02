param(
    [string]$SourceAdm = "",
    [string]$OpenJoc = "D:\Programs\OpenJOC\target\debug\openjoc.exe"
)

$ErrorActionPreference = "Stop"
$script = Join-Path $PSScriptRoot "run-decision-engine.ps1"
& $script -SourceAdm $SourceAdm -OpenJoc $OpenJoc
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
