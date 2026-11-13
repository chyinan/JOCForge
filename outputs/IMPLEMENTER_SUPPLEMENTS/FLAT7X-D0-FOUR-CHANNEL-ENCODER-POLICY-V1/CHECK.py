from __future__ import annotations

import hashlib
import json
from pathlib import Path

PACKAGE_DIR = Path(__file__).resolve().parent
REPO_ROOT = PACKAGE_DIR.parents[2]
PACKAGE_ID = "FLAT7X-D0-FOUR-CHANNEL-ENCODER-POLICY-V1"


def fail(message: str) -> None:
    raise SystemExit(f"PACKAGE_CHECK=FAIL {message}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_json(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except Exception as error:
        fail(f"invalid JSON {path.name}: {error}")
    if not isinstance(value, dict):
        fail(f"expected JSON object: {path.name}")
    return value


manifest_path = PACKAGE_DIR / "MANIFEST.json"
seal_path = PACKAGE_DIR / "PACKAGE-SEAL.json"
manifest = load_json(manifest_path)
seal = load_json(seal_path)
policy = load_json(PACKAGE_DIR / "POLICY.json")
vectors = load_json(PACKAGE_DIR / "INPUT-ONLY-VECTORS.json")
references = load_json(PACKAGE_DIR / "SOURCE-REFERENCES.json")

if manifest.get("package_id") != PACKAGE_ID or seal.get("package_id") != PACKAGE_ID:
    fail("package identity mismatch")
if manifest.get("schema") != "wash-flat7x-d0-encoder-policy-manifest.v1":
    fail("manifest schema mismatch")
if seal.get("schema") != "wash-flat7x-d0-encoder-policy-seal.v1":
    fail("seal schema mismatch")
if policy.get("contract_status") != "PROVISIONAL_IMPLEMENTATION_CONTRACT":
    fail("policy contract status mismatch")
if policy.get("contract_status") != manifest.get("status"):
    fail("manifest/policy status mismatch")
if policy.get("target", {}).get("coded_channel_locations") != ["Ls", "Rs", "Lrs", "Rrs"]:
    fail("D0 channel role order mismatch")
if policy.get("target", {}).get("chanmap") != "0x1A00":
    fail("D0 channel map mismatch")
if policy.get("target", {}).get("audio_block_count") != 6:
    fail("D0 block count mismatch")
if policy.get("encoder_policy", {}).get("exponent", {}).get("u7_groups_per_channel_block") != 84:
    fail("D15 U7 group count mismatch")
if policy.get("encoder_policy", {}).get("bit_allocation", {}).get("d0_provider", {}).get("active_bins_per_channel") != 253:
    fail("D0 BAP provider extent mismatch")
if policy.get("encoder_policy", {}).get("bit_allocation", {}).get("d0_provider", {}).get("reuse_existing_fbw217_lfe7_provider") is not False:
    fail("incompatible 217-bin BAP provider selected")
if policy.get("encoder_policy", {}).get("emdf", {}).get("container_length_bytes_max") != 511:
    fail("EMDF limit mismatch")
if policy.get("implementation_status", {}).get("production_allowed") is not False:
    fail("production status changed")
if vectors.get("execution_status") != "NOT_RUN" or vectors.get("raw_adm_pcm_payload_or_key_included") is not False:
    fail("vector scope changed")
if not any(row.get("id") == "bap_provider_extent" for row in vectors.get("vectors", [])):
    fail("D0 253-bin provider vector missing")
if not any(row.get("id") == "eof_pair_mismatch_reject" for row in vectors.get("vectors", [])):
    fail("I0/D0 EOF alignment vector missing")
if policy.get("eof_policy", {}).get("pair_receipt_rule") is None:
    fail("I0/D0 EOF receipt alignment rule missing")
if references.get("base_isolated_commit") != manifest.get("base_isolated_commit"):
    fail("base commit mismatch")

manifest_rows = manifest.get("files")
if not isinstance(manifest_rows, list) or not manifest_rows:
    fail("manifest file list missing")
manifest_file_map = {}
for row in manifest_rows:
    if not isinstance(row, dict) or not isinstance(row.get("path"), str) or not isinstance(row.get("sha256"), str):
        fail("malformed package file row")
    path = PACKAGE_DIR / row["path"]
    if not path.is_file() or digest(path) != row["sha256"]:
        fail(f"package file hash mismatch: {row.get('path')}")
    manifest_file_map[row["path"]] = row["sha256"]

actual_content_files = {
    path.relative_to(PACKAGE_DIR).as_posix()
    for path in PACKAGE_DIR.iterdir()
    if path.is_file() and path.name not in {"MANIFEST.json", "PACKAGE-SEAL.json"}
}
if set(manifest_file_map) != actual_content_files:
    fail("unlisted or missing package content files")

seal_files = seal.get("files")
if not isinstance(seal_files, dict):
    fail("seal file map missing")
actual_seal_files = {
    path.name
    for path in PACKAGE_DIR.iterdir()
    if path.is_file() and path.name != "PACKAGE-SEAL.json"
}
if set(seal_files) != actual_seal_files:
    fail("seal has unlisted or missing files")
for name, expected in seal_files.items():
    path = PACKAGE_DIR / name
    if not path.is_file() or digest(path) != expected:
        fail(f"seal file hash mismatch: {name}")
if seal.get("manifest_sha256") != digest(manifest_path):
    fail("manifest seal digest mismatch")

approved_packages = references.get("approved_wash_packages")
if not isinstance(approved_packages, list) or not approved_packages:
    fail("approved Wash package references missing")
checked_inputs = 0
for package in approved_packages:
    package_root = REPO_ROOT / package["path"]
    manifest_file = package_root / package["manifest_file"]
    if not manifest_file.is_file() or digest(manifest_file) != package["manifest_sha256"]:
        fail(f"referenced manifest changed: {package['package_id']}")
    seal_file = package.get("seal_file")
    if seal_file:
        path = package_root / seal_file
        if not path.is_file() or digest(path) != package["seal_sha256"]:
            fail(f"referenced seal changed: {package['package_id']}")
    if package.get("reviewed_normal_and_python_O") != "PASS":
        fail(f"referenced package checks not recorded PASS: {package['package_id']}")
    for relative, expected in package.get("relevant_files", []):
        path = package_root / relative
        if not path.is_file() or digest(path) != expected:
            fail(f"referenced evidence changed: {package['package_id']}/{relative}")
        checked_inputs += 1

if len(references.get("external_standards", [])) != 2:
    fail("external ETSI edition references incomplete")
excluded = references.get("excluded_materials", [])
if not any(row.get("package_id") == "CODEC-PUBLIC-BAP-PROVIDER-V1" for row in excluded):
    fail("incompatible internal BAP provider exclusion missing")
excluded_provider = next(row for row in excluded if row.get("package_id") == "CODEC-PUBLIC-BAP-PROVIDER-V1")
excluded_root = REPO_ROOT / excluded_provider["path"]
for relative, key in (("MANIFEST.json", "manifest_sha256"), ("DELIVERY.json", "delivery_sha256"), ("public-bap-48k-v1.json", "provider_sha256")):
    path = excluded_root / relative
    if not path.is_file() or digest(path) != excluded_provider[key]:
        fail(f"excluded provider reference changed: {relative}")
provider_meta = load_json(excluded_root / "public-bap-48k-v1.json")
if provider_meta.get("public_or_third_party_distribution") != "NOT_AUTHORIZED":
    fail("excluded provider license boundary changed")
if provider_meta.get("selected_profile") != "48000_HZ_ORDINARY_FBW217_LFE7_NO_COUPLING_DELTA_AHT":
    fail("excluded provider extent changed")
if not all(row.get("edition_date") for row in references["external_standards"]):
    fail("external standard edition missing")

print(
    "PACKAGE_CHECK=PASS "
    f"package={PACKAGE_ID} "
    f"files={len(manifest_rows)} "
    f"hash_bound_inputs={checked_inputs} "
    "normal_and_optimized_ready=true"
)




