from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DIALNORM-FRAME-U5-CONSUMER-BOUNDARY-V1"
SOURCE_MANIFEST = "f3b7064e86b97c7a06353553066038d9b1c782424e986cc735cf7b63975d6b7c"
STATUS = "ID12_FRAME_RECORD_NOT_CONSUMED_IN_SELECTED_BRANCH; CONVEXPSTR_WRITER_DISTINCT; FINAL_U5_JOIN_UNBOUND"


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
    require(evidence["selected_branch"]["record_pointer_use_after_save"] == "NONE_OBSERVED_IN_BOUNDED_BODY", "record-use boundary")
    require(evidence["first_bitfield_writer"]["five_bit_field"] == "convexpstr", "field identity")
    require(evidence["first_bitfield_writer"]["dialnorm_u5_join"] == "NOT_FOUND", "no U5 join")
    require(evidence["pcm_mismatch_relation"]["status"] == "UNBOUND", "PCM attribution")
    vector = load("INPUT-ONLY-VECTOR.json")
    require(vector["execution"] == "NOT_RUN" and vector["pcm_or_payload_included"] is False, "vector scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS selected record and convexpstr/DialNorm boundaries")


if __name__ == "__main__":
    main()
