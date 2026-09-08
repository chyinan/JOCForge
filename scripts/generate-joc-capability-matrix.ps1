# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts"
$generatedDate = Get-Date -Format "yyyy-MM-dd"

function Invoke-Checked([string]$file, [string[]]$arguments) {
    & $file @arguments
    if ($LASTEXITCODE -ne 0) { throw ("command failed with exit code {0}: {1} {2}" -f $LASTEXITCODE, $file, ($arguments -join ' ')) }
}

$compositionCsv = Join-Path $artifact "composition-capability-matrix.csv"
$compositionJson = Join-Path $artifact "composition-capability-matrix.json"
Invoke-Checked "cargo" @(
    "run", "-p", "jocforge", "--quiet", "--locked", "--", "composition-matrix",
    $compositionCsv, $compositionJson
)

# Preserve the established artifact names for callers while keeping one
# resolver-generated source of truth.
Copy-Item -LiteralPath $compositionCsv -Destination (Join-Path $artifact "joc-capability-matrix.csv") -Force
Copy-Item -LiteralPath $compositionJson -Destination (Join-Path $artifact "joc-capability-matrix.json") -Force

$rows = Import-Csv -LiteralPath $compositionCsv
$ceilingRows = @($rows | Where-Object { $_.verdict -eq "SEMANTIC_CEILING" })
$ceilingRows | Export-Csv -LiteralPath (Join-Path $artifact "multi-dependent-semantic-ceiling.csv") -NoTypeInformation -Encoding utf8
$ceilingRows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "multi-dependent-semantic-ceiling.json") -Encoding utf8

$verdictNames = @(
    "SUPPORTED_PASS",
    "PUBLIC_AMBIGUOUS",
    "NOT_APPLICABLE",
    "SEMANTIC_CEILING",
    "REJECTED_CAPACITY",
    "INVALID",
    "RESERVED",
    "SPEC_LEGAL_NOT_IMPLEMENTED"
)
$countMap = @{}
foreach ($verdict in $verdictNames) { $countMap[$verdict] = 0 }
foreach ($count in ($rows | Group-Object verdict)) { $countMap[$count.Name] = $count.Count }
$countLines = foreach ($verdict in $verdictNames) {
    "| $verdict | $($countMap[$verdict]) |"
}
$profileLines = foreach ($group in ($rows | Group-Object profile)) {
    $first = $group.Group | Select-Object -First 1
    $supported = @($group.Group | Where-Object { $_.verdict -eq "SUPPORTED_PASS" }).Count
    "| $($first.profile) | $($first.profile) | $supported |"
}

$markdown = @(
    "# JOC capability matrix",
    "",
    "> Generated on $generatedDate by the central JOCForge composition resolver. The complete lattice is in the resolver-generated CSV/JSON artifacts.",
    "",
    "## Central authority",
    "",
    "- CompositionRequest -> CompositionResolver -> JocEncodingPlan | CapabilityClassification is the only semantic classifier.",
    "- composition_capability_matrix() classifies profile, container, carriage, EMDF, partition, dependent count, LFE ownership, and bitrate without media encoding.",
    "- This compatibility script preserves the historical artifact filenames by copying the same resolver output; it contains no independent legality or capacity rules.",
    "",
    "## Profile summary",
    "",
    "| Profile | Index | SUPPORTED_PASS cells |",
    "|---|---|---:|",
    $profileLines,
    "",
    "## Status counts",
    "",
    "| Status | Cells |",
    "|---|---:|",
    $countLines,
    "",
    "## Evidence and boundaries",
    "",
    "- PUBLIC_AMBIGUOUS is reserved for mixed short partitions whose programme grouping and ownership are not publicly closed.",
    "- NOT_APPLICABLE keeps CMAF six-block I0 plus optional D0 and fixed legacy AC-3 framing separate from raw composition.",
    "- SEMANTIC_CEILING is distinct from capacity rejection: raw D6/D7 IDs exist, but the canonical Table-47 ownership graph has no additional non-duplicated slot.",
    "- SPEC_LEGAL_NOT_IMPLEMENTED is zero in the regenerated matrix; any future nonzero row is a release-blocking implementation or audit gap.",
    "- FFmpeg interoperability is metadata, not normative classification.",
    "",
    "See docs/orthogonal-composition-closure.md for the human-scale composition rules and targeted validation evidence.",
    ""
)
$markdown | Set-Content -LiteralPath (Join-Path $root "docs\joc-capability-matrix.md") -Encoding utf8

$ceilingMarkdown = @(
    "# Raw multi-dependent semantic ceiling",
    "",
    "> Generated on $generatedDate from the central JOCForge composition resolver.",
    "",
    "The canonical raw JOC plan exposes D0..D4 for idx0/idx3 and D0..D5 for idx1/idx2/idx4. D6/D7 remain syntactically legal raw E-AC-3 IDs but are SEMANTIC_CEILING for this ownership graph.",
    "",
    "Total semantic-ceiling cells: $($ceilingRows.Count).",
    "",
    "The complete rows are in artifacts/multi-dependent-semantic-ceiling.csv and .json; the multidimensional source is artifacts/composition-capability-matrix.csv and .json.",
    ""
)
$ceilingMarkdown | Set-Content -LiteralPath (Join-Path $root "docs\multi-dependent-semantic-ceiling.md") -Encoding utf8
Write-Output "JOC capability matrix written from central resolver to $artifact"
