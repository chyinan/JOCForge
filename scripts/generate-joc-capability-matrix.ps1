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
$carriages = @("etsi", "i0-skipfld", "legacy-ac3")
$emdfProfiles = @("etsi", "deployed")
$blockPartitions = @("1", "2", "3", "6")
$dependentTopologies = @("D0", "D0..D1", "D0..D2", "D0..D3", "D0..D4", "D0..D5", "D0..D6", "D0..D7")
$bitrates = @(384, 448, 512, 576, 640, 768, 896, 1024, 1088, 1152, 1216, 1280, 1408, 1536, 1664, 2304, 2944, 3584, 4224, 4864, 5504)
$scopes = @("raw-elementary", "CMAF")

function Get-DependentCount([string]$topology) {
    if ($topology -eq "D0") { return 1 }
    return ([int]($topology -replace "D0..D", "") + 1)
}

function Get-CanonicalDependentMaximum([int]$profileIndex) {
    if ($profileIndex -in @(0, 3)) { return 5 }
    return 6
}

function Get-CellStatus($profile, [string]$lfe, [string]$carriage, [string]$emdf, [string]$blocks, [string]$dependents, [int]$bitrate, [string]$scope) {
    $dependentCount = Get-DependentCount $dependents
    $canonicalMaximum = Get-CanonicalDependentMaximum $profile.index
    if ($profile.index -ge 5) { return "RESERVED" }
    if ($scope -eq "CMAF") {
        if ($carriage -ne "etsi" -or $emdf -ne "etsi" -or $blocks -ne "6" -or $dependents -ne "D0") {
            return "OUT_OF_SCOPE_CONTAINER"
        }
        if ($profile.index -le 4 -and $lfe -eq "absent" -and $bitrate -in @(1024, 1280, 1408, 1536, 1664)) {
            return "SUPPORTED_PASS"
        }
        return "SPEC_LEGAL_NOT_IMPLEMENTED"
    }
    if ($emdf -eq "deployed") { return "PUBLIC_AMBIGUOUS" }
    if ($carriage -eq "legacy-ac3") {
        if ($profile.index -gt 4) { return "RESERVED" }
        if ($bitrate -notin @(1024, 1088, 1152, 1216, 1280)) { return "REJECTED_CAPACITY" }
        if ($lfe -eq "present" -or $blocks -ne "6" -or $dependents -ne "D0") {
            return "SPEC_LEGAL_NOT_IMPLEMENTED"
        }
        if ($profile.index -ne 1 -and $bitrate -ne 1280) {
            return "SPEC_LEGAL_NOT_IMPLEMENTED"
        }
        return "SUPPORTED_PASS"
    }
    if ($carriage -eq "i0-skipfld") {
        if ($profile.index -ne 0) { return "PUBLIC_AMBIGUOUS" }
        if ($bitrate -ne 1024) { return "REJECTED_CAPACITY" }
        if ($lfe -eq "present") { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
        if ($blocks -ne "6" -or $dependents -ne "D0") { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
        return "SUPPORTED_PASS"
    }
    if ($blocks -ne "6") { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
    if ($dependentCount -gt $canonicalMaximum) { return "PUBLIC_LEGAL_NOT_IMPLEMENTED" }
    if ($lfe -eq "present") { return "SPEC_LEGAL_NOT_IMPLEMENTED" }
    $requiredBitrate = 1024 + (($dependentCount - 1) * 640)
    $carrierBitrate = $bitrate - ($dependentCount * 640)
    if ($bitrate -lt $requiredBitrate -or $carrierBitrate -notin @(384, 448, 512, 576, 640, 768, 896, 1024)) { return "REJECTED_CAPACITY" }
    if ($dependentCount -eq 1 -and $bitrate -in @(1024, 1280, 1408, 1536, 1664)) { return "SUPPORTED_PASS" }
    if ($bitrate -eq $requiredBitrate) { return "SUPPORTED_PASS" }
    return "SPEC_LEGAL_NOT_IMPLEMENTED"
}

$rows = [System.Collections.Generic.List[object]]::new()
foreach ($profile in $profiles) {
    foreach ($lfe in $lfeStates) {
        foreach ($carriage in $carriages) {
            foreach ($emdf in $emdfProfiles) {
                foreach ($blocks in $blockPartitions) {
                    foreach ($dependents in $dependentTopologies) {
                        $dependentCount = Get-DependentCount $dependents
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
                                    dependent_count = Get-DependentCount $dependents
                                    bitrate_kbps = $bitrate
                                    scope = $scope
                                    phase_signaled = $profile.phase
                                    public_legality = if ($profile.index -ge 5) { "RESERVED" } elseif ($scope -eq "CMAF" -and $dependents -ne "D0") { "INVALID" } elseif ($emdf -eq "deployed") { "PUBLIC_AMBIGUOUS" } else { "PUBLIC_LEGAL" }
                                    status = Get-CellStatus $profile $lfe $carriage $emdf $blocks $dependents $bitrate $scope
                                    ffmpeg_interoperability = if ($scope -eq "raw-elementary" -and $carriage -eq "etsi" -and $emdf -eq "etsi" -and $blocks -eq "6" -and $lfe -eq "absent" -and $dependentCount -gt 1 -and $bitrate -eq (1024 + (($dependentCount - 1) * 640))) { "UNSUPPORTED_DECODER_FEATURE" } elseif ($scope -eq "raw-elementary" -and $carriage -eq "etsi" -and $emdf -eq "etsi" -and $blocks -eq "6" -and $lfe -eq "absent" -and $dependentCount -eq 1 -and $bitrate -in @(1024,1280,1408,1536,1664)) { "PASS" } else { "NOT_MEASURED" }
                                    evidence = if ($profile.index -le 4 -and $scope -eq "CMAF" -and $lfe -eq "absent" -and $bitrate -in @(1024,1280,1408,1536,1664) -and $carriage -eq "etsi" -and $emdf -eq "etsi" -and $blocks -eq "6" -and $dependents -eq "D0") { "artifacts/cmaf-closure/validation.json" } elseif ($scope -eq "raw-elementary" -and $carriage -eq "etsi" -and $emdf -eq "etsi" -and $blocks -eq "6" -and $lfe -eq "absent" -and $dependentCount -le (Get-CanonicalDependentMaximum $profile.index) -and (($dependentCount -eq 1 -and $bitrate -in @(1024,1280,1408,1536,1664)) -or $bitrate -eq (1024 + (($dependentCount - 1) * 640)))) { "artifacts/multi-dependent-closure/validation.json" } elseif ($profile.index -eq 1 -and $carriage -eq "legacy-ac3" -and $emdf -eq "etsi" -and $blocks -eq "6" -and $dependents -eq "D0" -and $bitrate -in @(1024,1088,1152,1216,1280) -and $scope -eq "raw-elementary") { "artifacts/legacy-core-rate-sweep/validation.json" } elseif ($profile.index -le 4 -and $carriage -eq "legacy-ac3" -and $emdf -eq "etsi" -and $blocks -eq "6" -and $dependents -eq "D0" -and $bitrate -eq 1280 -and $scope -eq "raw-elementary") { "artifacts/legacy-core-profile-closure/validation.json" } elseif ($profile.index -le 4 -and $carriage -eq "etsi" -and $emdf -eq "etsi" -and $blocks -eq "6" -and $dependents -eq "D0" -and $bitrate -eq 1664 -and $scope -eq "raw-elementary") { "artifacts/profile-closure/profile-validation.json; artifacts/profile-closure/lfe-validation.json; artifacts/profile-closure/source-causality.json" } else { "docs/joc-capability-matrix.md" }
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
    "- ``SUPPORTED_PASS`` includes the canonical raw-elementary ETSI dependent carriage, strict EMDF, six-block cells evidenced by ``artifacts/multi-dependent-closure/validation.json``; the I0+D0 profile closure remains evidenced by ``artifacts/profile-closure/``.",
    "- ``REJECTED_CAPACITY`` is the deterministic lower-rate result for the current paired I0/D0 writer budget.",
    "- ``PUBLIC_AMBIGUOUS`` covers deployed codecdatae deviation and non-idx0 skip-field carriage where the public documents do not close the JOC-carrier meaning.",
    "- ``PUBLIC_LEGAL_NOT_IMPLEMENTED`` is a legal raw E-AC-3 dependent count not yet closed by the canonical no-LFE Table-47 JOC ownership plan; it is not an invalidity claim. ``SPEC_LEGAL_NOT_IMPLEMENTED`` covers other legal forms not yet generated, including source-derived short blocks and LFE-present multi-dependent plans.",
    "- ``OUT_OF_SCOPE_CONTAINER`` keeps CMAF's six-block, I0 plus optional D0, and container-specific signaling rules separate from raw elementary-stream support.",
    "- The advanced CMAF writer closes the no-LFE, strict, one-I0+D0, six-block, 1024/1280/1408/1536/1664 kbps idx0..idx4 cells; remaining CMAF cells stay explicitly unimplemented or out of scope.",
    "- `legacy-ac3` is supported for idx1 across 1024/1088/1152/1216/1280 kbps and for idx0..idx4 at 1280 kbps, with strict EMDF, no LFE, six blocks, and I0+D0; other legacy-core cells remain fail-closed.",
    "- idx5–idx7 are ``RESERVED`` and are never emitted. Raw D1+ cells with normative gates passed report FFmpeg interoperability separately as ``UNSUPPORTED_DECODER_FEATURE``; FFmpeg is not a legality gate.",
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
