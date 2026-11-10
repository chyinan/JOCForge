import hashlib
import json
from pathlib import Path


def fail(message: str) -> None:
    raise SystemExit(message)


root = Path(__file__).resolve().parent
manifest_bytes = (root / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads((root / "PACKAGE-SEAL.json").read_bytes())
if manifest.get("package_id") != "OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2":
    fail("package id mismatch")
if seal.get("package_id") != manifest.get("package_id"):
    fail("seal package id mismatch")
if seal.get("manifest_sha256") != hashlib.sha256(manifest_bytes).hexdigest():
    fail("manifest digest mismatch")
files = manifest.get("files")
if not isinstance(files, list) or not files:
    fail("manifest files missing")
if seal.get("file_paths") != [item.get("path") for item in files]:
    fail("seal path list mismatch")
for item in files:
    rel = Path(item["path"])
    if rel.is_absolute() or ".." in rel.parts:
        fail("unsafe relative path")
    payload = root / rel
    if not payload.is_file():
        fail(f"missing file: {rel}")
    data = payload.read_bytes()
    if len(data) != item["bytes"]:
        fail(f"size mismatch: {rel}")
    if hashlib.sha256(data).hexdigest() != item["sha256"]:
        fail(f"hash mismatch: {rel}")

ref = json.loads((root / "REFERENCE-STRUCTURE-ADDENDUM.json").read_bytes())
receipt = json.loads((root / "SOURCE-RECEIPTS.json").read_bytes())
if ref.get("production_allowed") is not False:
    fail("production must remain disabled")
if ref.get("full_track", {}).get("access_units") != 23033:
    fail("full-track AU count mismatch")
if ref.get("full_track", {}).get("joc_dmx_config_idx", {}).get("value") != 1:
    fail("profile index mismatch")
if ref.get("full_track", {}).get("table56", {}).get("etsi_strict") is not False:
    fail("strict disposition mismatch")
if ref.get("target_comparison", {}).get("target_encode_validation") != "NOT_EVALUATED":
    fail("target validation must remain not evaluated")
if receipt.get("raw_media_or_payload_included") is not False:
    fail("raw media/payload must remain excluded")

parent = root.parent / "OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1"
parent_manifest = parent / "MANIFEST.json"
parent_seal = parent / "PACKAGE-SEAL.json"
if not parent_manifest.is_file() or not parent_seal.is_file():
    fail("parent sanitized package missing")
if hashlib.sha256(parent_manifest.read_bytes()).hexdigest() != "f7f8630c3f18e990324f013e70486c561de6816a844f852f18f03a82ce28fbfc":
    fail("parent manifest mismatch")
if hashlib.sha256(parent_seal.read_bytes()).hexdigest() != "2d883e278bb83fa0d617610f94b8a87fa5849ac7bf19bdf08818d8d30d50740e":
    fail("parent seal mismatch")

print("PASS official demo JOC index addendum; profile index and validation status are distinct")
