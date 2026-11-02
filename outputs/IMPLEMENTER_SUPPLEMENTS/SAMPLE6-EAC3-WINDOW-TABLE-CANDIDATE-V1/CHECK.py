from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-EAC3-WINDOW-TABLE-CANDIDATE-V1"
SOURCE_MANIFEST = "f568d4f1990dbc5e6ea2e41a2eddf750b7a24f87cf62cdec53d1b2fce41c9e75"
STATUS = "STATIC_TRANSFORM_TABLE_SPLIT_FOUND; TRIAL_CANDIDATE_READY; PCM_CAUSALITY_UNBOUND"
EXPECTED_MISMATCHES = [
    (21,"3c0ea5aa","3c0ea5a9"),(24,"3c3f1258","3c3f1257"),(27,"3c7a15c9","3c7a15c8"),
    (34,"3cda168e","3cda168d"),(57,"3dd3d29b","3dd3d29c"),(74,"3e55acf8","3e55acf9"),
    (76,"3e653241","3e653242"),(80,"3e82f0b5","3e82f0b6"),(89,"3eab43c5","3eab43c6"),
    (93,"3ebe7ba1","3ebe7ba2"),(105,"3efafe35","3efafe36"),(194,"3f7e03d4","3f7e03d3")
]


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> None:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID and manifest["status"] == STATUS, "manifest id/status")
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST, "source manifest")
    entries = {entry["file"]: entry for entry in manifest["files"]}
    actual = {
        path.name for path in ROOT.iterdir()
        if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
    }
    require(actual == set(entries), "payload inventory")
    for name, entry in entries.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID, "seal id")
    require(set(sealed) == set(entries) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")

    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    table = evidence["window_difference"]
    observed = [(item["index"],item["native_f32_bits"],item["rust_f32_bits"]) for item in table["mismatches"]]
    require(table["word_count"] == 256 and table["one_ulp_mismatch_count"] == 12, "window counts")
    require(observed == EXPECTED_MISMATCHES, "mismatch list")
    require(evidence["candidate_trial"]["ready"] is True, "trial candidate")
    require(evidence["candidate_trial"]["must_not_do"][0].startswith("switch to the legacy table"), "legacy route caveat")
    require(evidence["candidate_trial"]["must_not_do"][1].startswith("claim this is the proven cause"), "causality caveat")
    require(evidence["execution_scope"]["dee_run"] == "NOT_RUN", "DEE scope")
    require(evidence["execution_scope"]["pcm_or_media_read"] is False, "PCM scope")
    require(evidence["pcm_context"]["causal_link_to_window_table_split"] == "UNBOUND", "PCM causality")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector["input"]["nonzero_words"] == [{"index":234,"f32_bits_hex":"3f800000","value":1.0}], "one-hot vector")
    require(vector["expected_fold_bits"]["native_primary_row_107"] == "3c0ea5aa", "native fixture output")
    require(vector["expected_fold_bits"]["rust_primary_row_107"] == "3c0ea5a9", "Rust fixture output")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS 12 one-ULP constants, selected provider scope, and PCM/root-cause boundaries")


if __name__ == "__main__":
    main()
