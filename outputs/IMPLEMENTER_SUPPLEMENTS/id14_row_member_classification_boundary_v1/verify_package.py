from __future__ import annotations
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    manifest_path = ROOT / "MANIFEST.json"
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes)
    require(manifest.get("package_id") == "id14_row_member_classification_boundary_v1", "package id")
    require(manifest.get("implementation_delivery") == "ALLOWED_WITHIN_DECLARED_SCOPE_ONLY", "delivery scope")
    source = json.loads((ROOT / "SOURCE-BINDING.json").read_text(encoding="utf-8"))
    require(source.get("incoming_manifest_sha256") == "3d5df8de998d0de67405ecc36b1b31b4c9a7bddc95b4b6bd185ab04eae67a3f4", "incoming manifest")
    require(source.get("parent_consumer_package_manifest_sha256") == "b6b56cfdbcf29a870496d87f04dc34b95d8a7a59db66f801b41cefb6e2dfec7b", "parent package")
    require(source.get("incoming_payload_sha256", {}).get("EVIDENCE.json") == "f0d8a5122997fb625420aa8444fbdcbee3f96382250debcecd49ce8f1d307026", "incoming evidence hash")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    roles = evidence.get("member_roles", {})
    require(roles.get("byte_plus3") == "read by a type-classification helper", "byte role")
    require(roles.get("u16_plus4") == "read only in a run-comparison branch", "U16 role")
    require(roles.get("plus4_alone_causes_normalized_value") == "NOT_CLAIMED", "causal overclaim")
    require(roles.get("semantic_name") == "UNBOUND", "semantic overclaim")
    require(evidence.get("count_context", {}).get("u6_writer_instruction_executed") is False, "writer status")
    require(evidence.get("limits", {}).get("output_parse") == "NOT_RUN", "output status")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(row.get("expected_output") == "NOT_SUPPLIED" for row in vectors.get("vectors", [])), "vector expectation")
    for entry in manifest.get("payloads", []):
        name = entry.get("name")
        require(isinstance(name, str) and Path(name).name == name, "unsafe payload name")
        path = (ROOT / name).resolve()
        require(path.parent == ROOT.resolve() and path.is_file(), "payload path")
        require(path.stat().st_size == entry.get("bytes") and sha256(path) == entry.get("sha256"), "payload hash")
    seal = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(seal and seal[0] == sha256(manifest_path), "manifest seal")
    verification = json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(verification.get("package_integrity") == "PASS_PACKAGE_INTEGRITY_ONLY", "verification scope")
    print("PASS: row member roles, semantic UNBOUND, source hashes, and writer boundary verified")

if __name__ == "__main__":
    main()
