from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-F15-STAGING-TRANSACTION-CONTRACT-V1"
PAYLOADS = (
    "IMPLEMENTER-HANDOFF.md",
    "SOURCE-RECEIPTS.json",
    "INPUT-ONLY-VECTORS.json",
    "CHECK.py",
)


def read_json(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def main() -> int:
    manifest = read_json("MANIFEST.json")
    seal = read_json("PACKAGE-SEAL.json")
    sources = read_json("SOURCE-RECEIPTS.json")
    vectors = read_json("INPUT-ONLY-VECTORS.json")

    require(manifest["package_id"] == PACKAGE_ID, "package identity")
    require(manifest["schema"] == "wash-sanitized-supplement.v1", "manifest schema")
    require(manifest["production_allowed"] is False, "manifest production gate")
    files = manifest.get("files")
    require(isinstance(files, list), "manifest files list")
    entries = {entry.get("path"): entry for entry in files if isinstance(entry, dict)}
    require(set(entries) == set(PAYLOADS), "payload list")
    for name in PAYLOADS:
        payload = (ROOT / name).read_bytes()
        entry = entries[name]
        require(entry.get("bytes") == len(payload), f"payload length: {name}")
        digest = hashlib.sha256(payload).hexdigest()
        require(entry.get("sha256") == digest, f"payload hash: {name}")

    manifest_hash = hashlib.sha256((ROOT / "MANIFEST.json").read_bytes()).hexdigest()
    require(seal["package_id"] == PACKAGE_ID, "seal identity")
    require(seal["manifest_sha256"] == manifest_hash, "manifest seal")
    require(sources["isolated_local_remote_match"] is True, "source ref match")
    require(sources["production_allowed"] is False, "production remains closed")
    require(vectors["vectors_are_runtime_observations"] is False, "vector tier")

    by_id = {item["id"]: item for item in vectors["cases"]}
    pair = by_id["pair_same_source_distinct_indices"]
    require(pair["source_generation_i0"] == pair["source_generation_d0"], "same-source pair")
    require(pair["published_au_index"] != pair["source_generation_i0"], "index domains separate")
    mismatch = by_id["pair_source_generation_mismatch"]
    require(mismatch["source_generation_i0"] != mismatch["source_generation_d0"], "mismatch vector")
    partial = by_id["partial_staging_write"]
    require(partial["bytes_written"] < partial["expected_au_bytes"], "partial-write vector")

    text = (ROOT / "IMPLEMENTER-HANDOFF.md").read_text(encoding="utf-8")
    require("UNBOUND" in text and "production_allowed=false" in text, "bounded disposition")
    print(f"PASS {PACKAGE_ID} manifest/seal, source receipts and input-only vectors")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
