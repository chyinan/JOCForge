import hashlib
import json
from pathlib import Path


def fail(message: str) -> None:
    raise SystemExit(message)


root = Path(__file__).resolve().parent
manifest_path = root / "MANIFEST.json"
seal_path = root / "PACKAGE-SEAL.json"
manifest_bytes = manifest_path.read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads(seal_path.read_bytes())

if manifest.get("schema") != "jocforge.implementer_supplement_manifest.v1":
    fail("unexpected manifest schema")
if manifest.get("package_id") != "OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V1":
    fail("unexpected package id")
if seal.get("package_id") != manifest.get("package_id"):
    fail("seal package id mismatch")
if seal.get("manifest_sha256") != hashlib.sha256(manifest_bytes).hexdigest():
    fail("manifest digest mismatch")

files = manifest.get("files")
if not isinstance(files, list) or not files:
    fail("manifest file list missing")
expected_paths = [item.get("path") for item in files]
if seal.get("file_paths") != expected_paths:
    fail("seal file path list mismatch")
if seal.get("seal_excluded_from_own_hash") is not True:
    fail("seal must be excluded from its own hash list")
for item in files:
    rel = Path(item["path"])
    if rel.is_absolute() or ".." in rel.parts:
        fail("unsafe package-relative path")
    payload = root / rel
    if not payload.is_file():
        fail(f"missing payload: {rel.as_posix()}")
    data = payload.read_bytes()
    if len(data) != item["bytes"]:
        fail(f"payload size mismatch: {rel.as_posix()}")
    if hashlib.sha256(data).hexdigest() != item["sha256"]:
        fail(f"payload hash mismatch: {rel.as_posix()}")

reference = json.loads((root / "REFERENCE-STRUCTURE.json").read_bytes())
source_receipts = json.loads((root / "SOURCE-RECEIPTS.json").read_bytes())
if reference.get("production_allowed") is not False:
    fail("production must remain disabled")
if reference.get("container", {}).get("packet_count") != 23033:
    fail("reference packet coverage mismatch")
if reference.get("container", {}).get("frame_order") != ["I0_LEGACY_INDEPENDENT", "D0_DEPENDENT"]:
    fail("reference frame order mismatch")
if reference.get("d0", {}).get("chanmap") != "0x1A00":
    fail("reference D0 chanmap mismatch")
if reference.get("joc_and_emdf", {}).get("d0_block2_skipfld_emdf_aus") != 23033:
    fail("reference D0 block-2 EMDF coverage mismatch")
if reference.get("target_comparison", {}).get("target_1664_emdf_fit") != "NOT_RUN":
    fail("target EMDF fit must remain not run")
if source_receipts.get("raw_media_or_payload_included") is not False:
    fail("raw media/payload must remain excluded")
if source_receipts.get("research_source_code_included") is not False:
    fail("research source code must remain excluded")
if source_receipts.get("mode21_artifact_included") is not False:
    fail("mode21 artifact must remain excluded")

print("PASS official demo reference structure package; full-track facts separated from target gates")
