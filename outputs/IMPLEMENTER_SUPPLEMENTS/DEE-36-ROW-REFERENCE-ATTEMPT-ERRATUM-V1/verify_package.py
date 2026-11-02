from __future__ import annotations
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PAYLOADS = ["REPORT.md", "EVIDENCE.json", "SOURCE-MANIFESTS.json", "verify_package.py", "seal_package.py"]
PACKAGE_ID = "DEE-36-ROW-REFERENCE-ATTEMPT-ERRATUM-V1"

def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")

def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()

def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    manifest_hash = digest(manifest_path)
    require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split() == [manifest_hash, "MANIFEST.json"], "manifest checksum")
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest hash")
    require(manifest.get("package_id") == PACKAGE_ID == seal.get("package_id"), "package id")
    require(manifest.get("files") == seal.get("files"), "manifest/seal file list")
    require([item.get("name") for item in manifest["files"]] == PAYLOADS, "payload order")
    for item in manifest["files"]:
        path = ROOT / item["name"]
        require(path.is_file(), f"missing {item['name']}")
        require(path.stat().st_size == item["bytes"], f"size {item['name']}")
        require(digest(path) == item["sha256"], f"hash {item['name']}")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(evidence["attempts"][0]["result"] == "INVALID_CONFIG; ACCEPTANCE_NOT_EVALUATED", "invalid 36 config disposition")
    require(evidence["attempts"][1]["result"] == "PROCESS_CRASH; PATH_BINDING_NOT_ESTABLISHED; ACCEPTANCE_NOT_EVALUATED", "first 36 path status")
    require(evidence["attempts"][2]["path_normalized_xml_equal"] is True, "38 exact XML")
    require(evidence["overall"]["36_row_acceptance"] == "NOT_ESTABLISHED", "36 status")
    require(evidence["overall"]["raw_payload_or_key_access"] is False, "payload/key boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS invalid 36 path attempt corrected; exact 38 crash remains not evaluated")

if __name__ == "__main__":
    main()
