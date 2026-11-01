import hashlib
import json
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit("FAIL: " + message)

def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()

def main():
    root = Path(__file__).resolve().parent
    manifest_path = root / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
    opened = json.loads((root / "OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(manifest["package_id"] == "id14_count_root_population_v1", "package id")
    require(manifest["implementation_delivery"] == "ALLOWED_WITHIN_DECLARED_SCOPE_ONLY", "delivery scope")
    require(evidence["status"] == manifest["status"], "status binding")
    require(evidence["normal_path"]["formulas"]["code13"] == "base + record_count", "code13 formula")
    require(evidence["normal_path"]["formulas"]["code14"] == "record_count", "code14 formula")
    require(evidence["normal_path"]["formulas"]["count_seed"] == "code13 - code14 = base", "count formula")
    require(evidence["normal_path"]["selector_rule"]["host_task_semantics"] == "UNBOUND", "selector boundary")
    require(evidence["open_edges"]["run_parent_and_input_generation"] == "UNBOUND", "generation boundary")
    require(evidence["open_edges"]["native_capture"] == "NOT_RUN", "capture status")
    require(evidence["exclusions"]["raw_decompilation"] is False, "decompilation exclusion")
    require(evidence["exclusions"]["private_key_or_license_contents_read"] is False, "secret exclusion")
    require(opened["package_id"] == manifest["package_id"], "open-edge package id")
    require({entry["name"] for entry in manifest["payloads"]} == {"README.md", "EVIDENCE.json", "OPEN-EDGES.json", "SOURCE-BINDING.json", "verify_package.py"}, "payload set")
    for entry in manifest["payloads"]:
        path = root / entry["name"]
        require(path.is_file(), "missing payload " + entry["name"])
        require(path.stat().st_size == entry["bytes"], "payload size " + entry["name"])
        require(sha256(path) == entry["sha256"], "payload hash " + entry["name"])
    seal = (root / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha256(manifest_path) == seal[0], "manifest seal")
    print("PASS: scoped ID14 code13/code14 semantics; selector and runtime generation remain open")

if __name__ == "__main__":
    main()
