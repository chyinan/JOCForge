from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)

def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main() -> None:
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    require(manifest.get("package_id") == "f06_bsi_owner_u16_conditional_writer_scope_v1", "package id mismatch")
    require(manifest.get("implementation_delivery") == "ALLOWED_WITHIN_DECLARED_SCOPE_ONLY", "delivery scope mismatch")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence.get("field", {}).get("storage_type") == "SIGNED_U16", "field type mismatch")
    require(evidence.get("field", {}).get("physical_unit_or_db_mapping") == "NOT_IDENTIFIED", "unit overclaim")
    writers = evidence.get("confirmed_setters", [])
    require(len(writers) == 2, "setter count mismatch")
    require({row.get("first_u16_at_owner_plus_0x34") for row in writers} == {0}, "conditional value mismatch")
    require(all(row.get("source_vector_read_by_assignment") is False for row in writers), "vector dependency overclaim")
    require(evidence.get("native_observation", {}).get("owner_constructor_first_write") == "NOT_LOCATED", "constructor state overclaim")
    require(evidence.get("native_observation", {}).get("active_guard") == "NOT_CAPTURED", "active guard overclaim")
    require(evidence.get("native_observation", {}).get("serializer_time_u16") == "NOT_CAPTURED", "runtime value overclaim")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(row.get("expected_output") == "NOT_SUPPLIED" for row in vectors.get("vectors", [])), "vector output invented")
    require(all(row.get("native_branch_observed") is False for row in vectors.get("conditional_write_inputs", [])), "branch observation overclaimed")
    source = json.loads((ROOT / "SOURCE-BINDING.json").read_text(encoding="utf-8"))
    require(source.get("incoming_package_manifest_sha256") == "62d0d2904475007a8a7de1b2965a830d60451aca5ab8307e951b902f1a47ef5b", "source manifest mismatch")
    for row in manifest.get("payloads", []):
        name = row.get("name")
        require(isinstance(name, str) and Path(name).name == name, "unsafe payload name")
        path = (ROOT / name).resolve()
        require(path.parent == ROOT.resolve() and path.is_file(), "payload path invalid")
        require(path.stat().st_size == row.get("bytes"), f"payload size mismatch: {name}")
        require(sha256(path) == row.get("sha256"), f"payload hash mismatch: {name}")
    seal = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(seal and seal[0] == sha256(ROOT / "MANIFEST.json"), "manifest seal mismatch")
    verification = json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(verification.get("package_integrity") in {"PENDING_PACKAGE_CHECK", "PASS_PACKAGE_INTEGRITY_ONLY"}, "verification scope mismatch")
    print("PASS: declared conditional writers, source scope, hashes, and UNBOUND boundaries verified")

if __name__ == "__main__":
    main()
