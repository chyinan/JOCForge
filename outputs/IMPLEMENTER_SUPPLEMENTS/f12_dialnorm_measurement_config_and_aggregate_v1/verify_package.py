from __future__ import annotations
import hashlib
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parent

def require(ok, message):
    if not ok:
        raise SystemExit(message)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    require(manifest.get("package_id") == "f12_dialnorm_measurement_config_and_aggregate_v1", "package id mismatch")
    require(manifest.get("implementation_delivery") == "ALLOWED_WITHIN_DECLARED_SCOPE_ONLY", "scope mismatch")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    native = evidence.get("native_measurement", {})
    require(native.get("selection") == "strict maximum; replace only when candidate > current aggregate", "aggregate selection mismatch")
    require(native.get("native_scalar_unit") == "NOT_STATED_IN_NATIVE_LOG_OR_STATIC_LABELS", "unit overclaim")
    config = evidence.get("configuration", {})
    require(config.get("meaning") == "measurement method selector, not loudness scalar", "metering mode meaning mismatch")
    limits = evidence.get("limits", {})
    require(limits.get("adm_programme_to_native_measurement_entry") == "NOT_PROVEN", "source join overclaim")
    require(limits.get("eac3_output_parse") == "NOT_RUN", "output status overclaim")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    require(all(row.get("expected_output") == "NOT_SUPPLIED" for row in vectors.get("vectors", [])), "vector output invented")
    for row in manifest.get("payloads", []):
        name = row.get("name")
        require(isinstance(name, str) and Path(name).name == name, "unsafe payload name")
        path = (ROOT / name).resolve()
        require(path.parent == ROOT.resolve() and path.is_file(), "payload path")
        require(path.stat().st_size == row.get("bytes") and digest(path) == row.get("sha256"), "payload hash or size")
    sidecar = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(sidecar and sidecar[0] == digest(ROOT / "MANIFEST.json"), "manifest seal")
    verification = json.loads((ROOT / "VERIFICATION.json").read_text(encoding="utf-8"))
    require(verification.get("package_integrity") in {"PENDING_PACKAGE_CHECK", "PASS_PACKAGE_INTEGRITY_ONLY"}, "verification state")
    print("PASS: measurement selector, float-max aggregation, source scope and open edges verified")

if __name__ == "__main__":
    main()
