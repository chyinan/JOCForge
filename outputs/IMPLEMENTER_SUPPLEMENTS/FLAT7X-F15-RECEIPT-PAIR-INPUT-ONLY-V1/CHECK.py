from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-F15-RECEIPT-PAIR-INPUT-ONLY-V1"
PAYLOADS = (
    "CHECK.py",
    "IMPLEMENTER-HANDOFF.md",
    "INPUT-ONLY-VECTORS.json",
    "SOURCE-RECEIPTS.json",
)


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def main() -> int:
    manifest = load("MANIFEST.json")
    seal = load("PACKAGE-SEAL.json")
    sources = load("SOURCE-RECEIPTS.json")
    vectors = load("INPUT-ONLY-VECTORS.json")

    require(manifest.get("package_id") == PACKAGE_ID, "package identity")
    require(manifest.get("schema") == "wash-sanitized-supplement.v1", "manifest schema")
    require(manifest.get("production_allowed") is False, "production gate")
    entries = manifest.get("files")
    require(isinstance(entries, list), "manifest file list")
    by_path = {entry.get("path"): entry for entry in entries if isinstance(entry, dict)}
    require(set(by_path) == set(PAYLOADS), "payload set")
    for name in PAYLOADS:
        data = (ROOT / name).read_bytes()
        entry = by_path[name]
        require(entry.get("bytes") == len(data), f"payload bytes: {name}")
        require(entry.get("sha256") == hashlib.sha256(data).hexdigest(), f"payload hash: {name}")

    manifest_hash = hashlib.sha256((ROOT / "MANIFEST.json").read_bytes()).hexdigest()
    require(seal.get("package_id") == PACKAGE_ID, "seal identity")
    require(seal.get("manifest_sha256") == manifest_hash, "manifest seal")
    require(sources.get("isolated_local_remote_match") is True, "source ref match")
    require(sources.get("production_allowed") is False, "source production gate")
    require(vectors.get("vector_status") == "SYNTHETIC_INPUT_ONLY_NOT_RUN", "vector status")
    require(vectors.get("raw_frame_bytes_included") is False, "frame bytes exclusion")
    require(vectors.get("raw_emdf_bytes_included") is False, "EMDF bytes exclusion")
    require(vectors.get("replay_status") == "NOT_RUN", "replay status")

    positive = vectors["positive_pair"]
    identity = positive["integrity_identity"]
    require(identity["published_au_index"] == 0, "AU index")
    require(positive["f13_metadata"]["identity_matches_integrity"] is True, "F13 identity")
    require(positive["f13_metadata"]["parsed_carrier_block"] == 2, "D0 carrier block")
    require(0 < positive["f13_metadata"]["protected_container_bytes"] <= 511, "EMDF bound")
    require(positive["i0_integrity_receipt"]["role"] == "IndependentI0", "I0 role")
    require(positive["d0_integrity_receipt"]["role"] == "DependentD0", "D0 role")
    require(positive["i0_integrity_receipt"]["frame_bytes"] == 2560, "I0 extent")
    require(positive["d0_integrity_receipt"]["frame_bytes"] == 4096, "D0 extent")
    require(positive["i0_integrity_receipt"]["identity_matches_f13_and_d0"] is True, "I0 identity")
    require(positive["d0_integrity_receipt"]["identity_matches_f13_and_i0"] is True, "D0 identity")
    require(positive["f15_candidate"]["i0_then_d0"] is True, "pair ordering")
    require(positive["f15_candidate"]["labels_match_within_pair"] is True, "pair label equality")
    require(positive["f15_candidate"]["label_join_to_runtime_callback_owner"] == "UNBOUND", "source owner join")
    require(positive["f15_candidate"]["production_allowed"] is False, "positive production gate")

    text = (ROOT / "IMPLEMENTER-HANDOFF.md").read_text(encoding="utf-8")
    require("NOT_RUN" in text and "UNBOUND" in text, "handoff boundary")
    print(f"PASS {PACKAGE_ID} manifest/seal, pair identity and input-only boundary")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
