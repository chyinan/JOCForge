// pattern: Functional Core
use jocforge_domain::spatial::rectangular_nnls::{
    NnlsRejectionReason, NnlsResult, NnlsStatus, solve_rectangular_nnls,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const PACKAGE_MANIFEST_SHA256: &str =
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a";
const SOURCE_VECTOR_SHA256: &str =
    "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8";
const SOURCE_SPEC_SHA256: &str = "7cfb51e8905f739e60bebcd30899887bda52cc493a5bab0c976ca645548b09fc";
const SOURCE_ADAPTER_SHA256: &str =
    "9bce2e35ce47147c76c13e58ee976d44c5109938f0b19e1f3a81f1d8a61fa1ae";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn selected_f03n_cases_match_complete_solution_and_active_set_diagnostics() {
    let manifest_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f01e_provider/MANIFEST.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let vectors_bytes =
        std::fs::read(crate_root().join("tests/fixtures/f04_component/unit-vectors.json")).unwrap();
    let vectors: Value = serde_json::from_slice(&vectors_bytes).unwrap();
    assert_eq!(sha256(&manifest_bytes), PACKAGE_MANIFEST_SHA256);
    assert_eq!(sha256(&vectors_bytes), SOURCE_VECTOR_SHA256);
    let source_entry = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["file"] == "test-vectors/unit/vectors.json")
        .unwrap();
    assert_eq!(source_entry["sha256"], SOURCE_VECTOR_SHA256);
    assert_eq!(source_entry["bytes"], vectors_bytes.len() as u64);
    for (path, expected_sha, local_path) in [
        (
            "specs/03-rectangular-nnls-initial-fit.md",
            SOURCE_SPEC_SHA256,
            crate_root().join("tests/fixtures/rectangular_nnls/source-spec.md"),
        ),
        (
            "test-vectors/unit/ADAPTER-CONTRACT.md",
            SOURCE_ADAPTER_SHA256,
            crate_root().join("tests/fixtures/rectangular_nnls/adapter-contract.md"),
        ),
    ] {
        let entry = manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["file"] == path)
            .unwrap();
        let bytes = std::fs::read(local_path).unwrap();
        assert_eq!(entry["sha256"], expected_sha);
        assert_eq!(sha256(&bytes), expected_sha);
        assert_eq!(
            entry["bytes"].as_u64().unwrap(),
            u64::try_from(bytes.len()).unwrap()
        );
    }

    let cases = vectors["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["functional_id"] == "F03N")
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 9);
    assert_eq!(
        cases
            .iter()
            .map(|case| case["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "U-OUT-NNLS-1",
            "U-OUT-NNLS-3",
            "U-OUT-NNLS-15",
            "U-OUT-NNLS-REMOVE",
            "U-OUT-NNLS-FRACTIONAL",
            "U-OUT-NNLS-RANK",
            "U-OUT-NNLS-NONPOSITIVE-TRIAL",
            "U-OUT-NNLS-LAST-POP",
            "U-OUT-NNLS-ITERATION-LIMIT",
        ]
    );
    for case in cases {
        let input = &case["input"];
        let matrix = input["context_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                row.as_array()
                    .unwrap()
                    .iter()
                    .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let response = input["response_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u32::try_from(word.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let actual = solve_rectangular_nnls(&matrix, &response)
            .unwrap_or_else(|error| panic!("{}: {error:?}", case["id"]));
        assert_result_matches(&actual, &case["expected"], case["id"].as_str().unwrap());
    }
}

fn assert_result_matches(actual: &NnlsResult, expected: &Value, case_id: &str) {
    let expected_status = match expected["status"].as_str().unwrap() {
        "SOLVED" => NnlsStatus::Solved,
        "ITERATION_LIMIT" => NnlsStatus::IterationLimit,
        other => panic!("unmapped F03N status {other}"),
    };
    assert_eq!(actual.status, expected_status, "{case_id}");
    assert_eq!(
        actual.seed_u32,
        expected["seed_u32"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| u32::try_from(value.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>(),
        "{case_id}"
    );
    assert_eq!(
        actual.solution_f64.len(),
        expected["solution_f64_le_hex"].as_array().unwrap().len(),
        "{case_id}"
    );
    for (solution, expected_hex) in actual
        .solution_f64
        .iter()
        .zip(expected["solution_f64_le_hex"].as_array().unwrap())
    {
        assert_eq!(
            f64_hex(*solution),
            expected_hex.as_str().unwrap(),
            "{case_id}"
        );
    }
    assert_eq!(
        actual.passive,
        usize_array(&expected["passive"]),
        "{case_id}"
    );
    assert_eq!(
        actual.inactive,
        usize_array(&expected["inactive"]),
        "{case_id}"
    );
    assert_eq!(
        actual.permutation,
        usize_array(&expected["permutation"]),
        "{case_id}"
    );
    assert_eq!(
        actual.accepted,
        usize_array(&expected["accepted"]),
        "{case_id}"
    );
    assert_eq!(
        actual.iterations,
        usize::try_from(expected["iterations"].as_u64().unwrap()).unwrap(),
        "{case_id}"
    );
    assert_eq!(
        actual.iteration_limit,
        usize::try_from(expected["iteration_limit"].as_u64().unwrap()).unwrap(),
        "{case_id}"
    );
    assert_eq!(
        actual.rejected.len(),
        expected["rejected"].as_array().unwrap().len(),
        "{case_id}"
    );
    for (actual, expected) in actual
        .rejected
        .iter()
        .zip(expected["rejected"].as_array().unwrap())
    {
        let reason = match expected["reason"].as_str().unwrap() {
            "RELATIVE_RANK" => NnlsRejectionReason::RelativeRank,
            "NONPOSITIVE_TRIAL" => NnlsRejectionReason::NonpositiveTrial,
            other => panic!("unmapped F03N rejection reason {other}"),
        };
        assert_eq!(
            actual.variable,
            usize::try_from(expected["variable"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(actual.reason, reason);
        assert_eq!(
            actual.passive_count,
            usize::try_from(expected["passive_count"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.rank_increment.to_bits(),
            expected["rank_increment"].as_f64().unwrap().to_bits()
        );
    }
    assert_eq!(
        actual.removals.len(),
        expected["removals"].as_array().unwrap().len(),
        "{case_id}"
    );
    for (actual, expected) in actual
        .removals
        .iter()
        .zip(expected["removals"].as_array().unwrap())
    {
        assert_eq!(
            actual.variable,
            usize::try_from(expected["variable"].as_u64().unwrap()).unwrap()
        );
        assert_eq!(
            actual.passive_position,
            usize::try_from(expected["passive_position"].as_u64().unwrap()).unwrap()
        );
    }
}

fn usize_array(value: &Value) -> Vec<usize> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| usize::try_from(value.as_u64().unwrap()).unwrap())
        .collect()
}

fn f64_hex(value: f64) -> String {
    let mut output = String::with_capacity(16);
    for byte in value.to_le_bytes() {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}
