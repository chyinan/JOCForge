# pattern: Imperative Shell

param()

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$artifact = Join-Path $root "artifacts"
$profiles = @(
    [pscustomobject]@{ index = 0; name = "5.X"; aliases = "idx0"; input_count = 5; carriers = "L,R,C,Ls,Rs"; topology = "I0+ D0"; phase = $false },
    [pscustomobject]@{ index = 1; name = "Flat-7.X"; aliases = "idx1"; input_count = 7; carriers = "L,R,C,Ls,Rs,Lrs,Rrs"; topology = "I0+ D0"; phase = $false },
    [pscustomobject]@{ index = 2; name = "5.X+2"; aliases = "idx2"; input_count = 7; carriers = "L,R,C,Ls,Rs,Tfl,Tfr"; topology = "I0+ D0"; phase = $false },
    [pscustomobject]@{ index = 3; name = "5.X Phase"; aliases = "idx3"; input_count = 5; carriers = "L,R,C,Ls,Rs"; topology = "I0+ D0"; phase = $true },
    [pscustomobject]@{ index = 4; name = "5.X+2 Phase"; aliases = "idx4"; input_count = 7; carriers = "L,R,C,Ls,Rs,Tfl,Tfr"; topology = "I0+ D0"; phase = $true },
    [pscustomobject]@{ index = 5; name = "reserved-5"; aliases = "idx5"; input_count = 0; carriers = ""; topology = "never"; phase = $false },
    [pscustomobject]@{ index = 6; name = "reserved-6"; aliases = "idx6"; input_count = 0; carriers = ""; topology = "never"; phase = $false },
    [pscustomobject]@{ index = 7; name = "reserved-7"; aliases = "idx7"; input_count = 0; carriers = ""; topology = "never"; phase = $false }
)
$lfeStates = @("absent", "present")
$carriages = @("etsi", "i0-skipfld")
$emdfProfiles = @("etsi", "deployed")
$blockPartitions = @("1", "2", "3", "6")
$dependentTopologies = @("D0", "D0..D7")
$bitrates = @(384, 448, 640, 768, 896, 1024, 1280, 1408, 1536, 1664)
$scopes = @("raw-elementary", "CMAF")

function Get-CellStatus($profile, [string]$lfe, [string]$carriage, [string]$emdf, [string]$blocks, [string]$dependents, [int]$bitrate, [string]$scope) {
    if ($profile.index -ge 5) { return "RESERVED" }
    if ($scope -eq "CMAF") {
        if ($carriage -ne "etsi" -or $emdf -ne "etsi" -or $blocks -ne "6" -or $dependents -ne "D0") {
            return "OUT_OF_SCOPE_CONTAINER"
        }
        return "SPEC_LEGAL_NOT_IMPLEMENTED"
    }
    if ($emdf -eq "deployed") { return "PUBLIC_AMBIGUOUS" }
    if ($carriage -eq "i0-skipfld") {
        if ($profile.index -ne 0) { return "PUBLIC_AMBIGUOUS" }
        if ($bitrate -ne 1024) { return "REJECTED_CAPACITY" }
        if ($lfe -eq "present") { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
        if ($blocks -ne "6" -or $dependents -ne "D0") { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
        return "SUPPORTED_PASS"
    }
    if ($blocks -ne "6") { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
    if ($dependents -ne "D0") { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
    if ($bitrate -lt 1024) { return "REJECTED_CAPACITY" }
    if ($lfe -eq "present" -and $bitrate -ne 1664) { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
    return "SUPPORTED_PASS"
}

$rows = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $profiles) {
    foreach ($lfe in $lfeStates) {
        foreach ($carriage in $carriages) {
            foreach ($emdf in $emdfProfiles) {
                foreach ($blocks in $blockPartitions) {
                    foreach ($dependents in $dependentTopologies) {
                        foreach ($bitrate in $bitrates) {
                            foreach ($scope in $scopes) {
                                $rows.Add([ordered]@{
                                    semantic_profile = $profile.name
                                    profile_index = $profile.index
                                    aliases = $profile.aliases
                                    lfe = $lfe
                                    carrier_input_count = $profile.input_count
                                    carrier_inputs = $profile.carriers
                                    carriage = $carriage
                                    emdf_profile = $emdf
                                    block_partition = $blocks
                                    dependent_topology = $dependents
                                    bitrate_kbps = $bitrate
                                    scope = $scope
                                    phase_signaled = $profile.phase
                                    status = Get-CellStatus $profile $lfe $carriage $emdf $blocks $dependents $bitrate $scope
                                    evidence = if ($profile.index -le 4 -and $carriage -eq "etsi" -and $emdf -eq "etsi" -and $blocks -eq "6" -and $dependents -eq "D0" -and $bitrate -eq 1664 -and $scope -eq "raw-elementary") { "artifacts/profile-closure/profile-validation.json; artifacts/profile-closure/lfe-validation.json; artifacts/profile-closure/source-causality.json" } else { "docs/joc-capability-matrix.md" }
                                })
                            }
                        }
                    }
                }
            }
        }
    }
}

$rows | Export-Csv -LiteralPath (Join-Path $artifact "joc-capability-matrix.csv") -NoTypeInformation -Encoding utf8
$rows | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifact "joc-capability-matrix.json") -Encoding utf8

$counts = $rows | ForEach-Object { $_.status } | Group-Object | Sort-Object Name
$summary = foreach ($count in $counts) { "| $($count.Name) | $($count.Count) |" }
$profileSummary = foreach ($profile in $profiles) {
    $supported = @($rows | Where-Object { $_.profile_index -eq $profile.index -and $_.status -eq "SUPPORTED_PASS" }).Count
    "| $($profile.name) | idx$($profile.index) | $($profile.carriers) | $supported |"
}

$markdown = @(
    "# JOC capability matrix",
    "",
    "> Generated from the public-spec closure rules on 2026-09-07. The CSV and JSON artifacts enumerate every cross-product cell; this document records the authority and the evidence boundary.",
    "",
    "## Profile authority",
    "",
    "| Semantic profile | Index | Source-derived inputs | Canonical carriage | Supported cells |",
    "|---|---:|---|---|---:|",
    $profileSummary,
    "",
    "idx0–idx4 follow ETSI TS 103 420 V1.2.1 Table 47/48. JOCForge uses `Lrs/Rrs` as the internal labels for the Table-47 `Lb/Rb` rear pair. idx3/idx4 are signaling-only phase variants; no extra PCM phase transform is applied.",
    "",
    "## Status counts",
    "",
    "| Status | Cells |",
    "|---|---:|",
    $summary,
    "",
    "## Evidence and boundaries",
    "",
    "- `SUPPORTED_PASS` currently means the canonical raw-elementary ETSI dependent carriage, strict EMDF, six-block, I0+D0, and one of the validated 1024/1280/1408/1536/1664 kbps cells. The 1664 kbps five-profile render/source-causality run and the matching LFE-present run are in `artifacts/profile-closure/`.",
    "- `REJECTED_CAPACITY` is the deterministic lower-rate result for the current paired I0/D0 writer budget.",
    "- `PUBLIC_AMBIGUOUS` covers deployed codecdatae deviation and non-idx0 skip-field carriage where the public documents do not close the JOC-carrier meaning.",
    "- `SPEC_LEGAL_NOT_IMPLEMENTED` covers public raw-E-AC-3 forms not yet generated by JOCForge, including short blocks and multiple dependents.",
    "- `OUT_OF_SCOPE_CONTAINER` keeps CMAF's six-block, I0 plus optional D0, and container-specific signaling rules separate from raw elementary-stream support.",
    "- idx5–idx7 are `RESERVED` and are never emitted.",
    "",
    "## Public references",
    "",
    "- [ETSI TS 103 420 V1.2.1](https://www.etsi.org/deliver/etsi_ts/103400_103499/103420/01.02.01_60/ts_103420v010201p.pdf), Tables 47/48, clauses 8.1–8.3, and Annex E.",
    "- [ETSI TS 102 366 V1.4.1](https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf), Table E.1.3 and clauses E.1.3/E.2.8.",
    "- Local public OpenJOC evidence: `D:/Programs/OpenJOC/crates/openjoc-joc/tests/stateful_decoder.rs`, `D:/Programs/OpenJOC/crates/openjoc-eac3/src/access_unit.rs`, and `D:/Programs/OpenJOC/docs/site/project/capabilities.md`.",
    ""
)
$markdown | Set-Content -LiteralPath (Join-Path $root "docs\joc-capability-matrix.md") -Encoding utf8
Write-Output "JOC capability matrix written to $artifact"
