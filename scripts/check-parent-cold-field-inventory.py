# Imperative Shell: verify inventory and fixture identities without mutating them.
"""Independently validate the machine-readable, non-normative v35 field inventory."""
import hashlib
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "crates/jocforge-domain/tests/fixtures/spatial_control_cold_serializer_blocker"
IMPLEMENTER_MANIFEST = FIXTURE / "SPATIAL-CONTROL-COLD-SERIALIZER-BLOCKER-MANIFEST.json"
IMPLEMENTER_MANIFEST_SHA256 = "71ca9bb30c2129019a76ecb703df47a88f5384582bb65a53faee92103e9dcd0b"
INVENTORY_PATH = ROOT / "docs/implementation/PARENT-COLD-FIELD-INVENTORY.json"
INVENTORY_MANIFEST_PATH = ROOT / "docs/implementation/PARENT-COLD-FIELD-INVENTORY-MANIFEST.json"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_pairs)


def resolve_pointer(document, pointer: str):
    value = document
    if pointer:
        if not pointer.startswith("/"):
            raise ValueError(f"invalid JSON pointer: {pointer}")
        for part in pointer[1:].split("/"):
            part = part.replace("~1", "/").replace("~0", "~")
            value = value[int(part)] if isinstance(value, list) else value[part]
    return value


def source_path(source_file: str, package_files: dict) -> Path:
    if source_file in package_files:
        return FIXTURE / source_file
    return ROOT / "docs/implementation" / source_file


def byte_coverage(entries, offset_getter, width_getter, start: int, length: int, label: str):
    actual = []
    for entry in entries:
        record = entry["native_observation"]["source_record"]
        offset = offset_getter(record)
        width = width_getter(record)
        actual.extend(range(offset, offset + width))
    expected = list(range(start, start + length))
    if sorted(actual) != expected:
        raise ValueError(f"{label} offset coverage mismatch: {sorted(actual)}")


implementer_bytes = IMPLEMENTER_MANIFEST.read_bytes()
if digest(implementer_bytes) != IMPLEMENTER_MANIFEST_SHA256:
    raise SystemExit("v35 implementer manifest hash mismatch")
implementer = json.loads(implementer_bytes.decode("utf-8"), object_pairs_hook=unique_pairs)
if implementer.get("identity") != "spatial-control-cold-serializer.blocked.v35":
    raise SystemExit("unexpected implementer package identity")
if len(implementer.get("files", [])) != 44:
    raise SystemExit("v35 package asset count mismatch")
admission = implementer["parent_admission"]
if admission != {
    "selected_gate1_reverse_dependencies": 0,
    "gate1_closed": False,
    "m1_canonical_schema_and_digest": "CLEAN_ROOM_DESIGN_REQUIRED",
    "m2_serializer_parser_cold_roundtrip": "NOT_RUN",
    "m3_receipt_transaction_vectors": "NOT_RUN",
}:
    raise SystemExit("unexpected v35 parent admission state")
if implementer["source_evidence"]["implementation_ready"] != "NO" or implementer["source_evidence"]["production_allowed"] is not False:
    raise SystemExit("v35 package cannot enable implementation or production")
package_files = {item["file"]: item for item in implementer["files"]}
if len(package_files) != 44 or len(package_files) != len(implementer["files"]):
    raise SystemExit("v35 package file list is not 44 unique assets")
for name, declaration in package_files.items():
    data = (FIXTURE / name).read_bytes()
    if len(data) != declaration["bytes"] or digest(data) != declaration["sha256"]:
        raise SystemExit(f"v35 package asset hash mismatch: {name}")

inventory_manifest = load_json(INVENTORY_MANIFEST_PATH)
inventory_bytes = INVENTORY_PATH.read_bytes()
if len(inventory_bytes) != inventory_manifest["inventory_bytes"] or digest(inventory_bytes) != inventory_manifest["inventory_sha256"]:
    raise SystemExit("field inventory artifact hash mismatch")
inventory = json.loads(inventory_bytes.decode("utf-8"), object_pairs_hook=unique_pairs)

if inventory["source_implementer_manifest"]["sha256"] != IMPLEMENTER_MANIFEST_SHA256:
    raise SystemExit("field inventory points at a different implementer package")
if inventory.get("identity") != "parent-cold-field-inventory.v35" or inventory["source_implementer_manifest"]["identity"] != implementer["identity"]:
    raise SystemExit("field inventory does not identify the v35 package")
if inventory["status"] != "INVENTORY_INCOMPLETE_NOT_NORMATIVE_SCHEMA" or inventory["expected_used_as_seed"] is not False:
    raise SystemExit("field inventory must remain incomplete and non-normative")
if inventory["implementation_ready"] != "NO" or inventory["production_allowed"] is not False:
    raise SystemExit("field inventory cannot enable implementation or production")

registered_sources = {source["file"]: source for source in inventory["source_assets"]}
if len(registered_sources) != len(inventory["source_assets"]):
    raise SystemExit("duplicate source asset in inventory registry")
for name, declaration in registered_sources.items():
    path = source_path(name, package_files)
    data = path.read_bytes()
    if len(data) != declaration["bytes"] or digest(data) != declaration["sha256"]:
        raise SystemExit(f"source hash mismatch: {name}")
    if name in package_files:
        package_declaration = package_files[name]
        if package_declaration["bytes"] != declaration["bytes"] or package_declaration["sha256"] != declaration["sha256"]:
            raise SystemExit(f"v35 manifest disagrees with source registry: {name}")

entries = inventory["field_entries"]
field_ids = [entry["inventory_id"] for entry in entries]
if len(field_ids) != len(set(field_ids)):
    raise SystemExit("duplicate inventory_id")
for entry in entries:
    source = entry["source"]
    declaration = registered_sources.get(source["asset"])
    if declaration is None or declaration["sha256"] != source["sha256"]:
        raise SystemExit(f"inventory row has unregistered source: {entry['inventory_id']}")
    source_document = load_json(source_path(source["asset"], package_files))
    source_record = resolve_pointer(source_document, source["json_pointer"])
    if source_record != entry["native_observation"]["source_record"]:
        raise SystemExit(f"source projection differs: {entry['inventory_id']}")
    if entry["clean_room_candidate"]["normative"] is not False:
        raise SystemExit(f"candidate disposition cannot be normative: {entry['inventory_id']}")
    if not entry["native_observation"].get("producer_stage") or not entry["native_observation"].get("consumer_stage"):
        raise SystemExit(f"missing producer/consumer stage: {entry['inventory_id']}")

actual_counts = Counter(entry["group"] for entry in entries)
if dict(actual_counts) != inventory["expected_group_counts_from_sources"]:
    raise SystemExit("field group counts disagree with inventory summary")

def group(name):
    return [entry for entry in entries if entry["group"] == name]


byte_coverage(group("descriptor"), lambda item: item["offset"], lambda item: item["bytes"], 0, 120, "descriptor")
byte_coverage(group("policy"), lambda item: item["offset"], lambda item: item["bytes"], 0, 44, "policy block")
byte_coverage(group("measurement_summary"), lambda item: item.get("descriptor_offset", item.get("offset")) - 72, lambda item: item.get("bytes", 4), 0, 44, "measurement summary")
byte_coverage(group("selected_prefix"), lambda item: item["offset"], lambda item: 4, 0, 104, "selected prefix")
byte_coverage(group("external_metadata_layout"), lambda item: item["offset"], lambda item: item["bytes"], 0, 120, "packed external metadata")
byte_coverage(group("channel_record"), lambda item: item["offset"], lambda item: item["bytes"], 0, 6, "channel record")
byte_coverage(group("trim_payload"), lambda item: item["offset"], lambda item: item["bytes"], 0, 24, "trim payload")

if actual_counts["ddp_dictionary"] != 27 or actual_counts["measurement_core_slots"] != 3:
    raise SystemExit("separate dictionary or core measurement slot inventory incomplete")
omitted = inventory["omitted_field_ledger"]
if len(omitted) != inventory_manifest["omitted_field_count"] or not omitted:
    raise SystemExit("omitted-field completeness ledger missing")
allowed_omitted_dispositions = {"EXCLUDED_CANDIDATE_SELECTED_MODE0_ONLY", "EXCLUDED_CANDIDATE_OUTSIDE_SELECTED_PROFILE", "EXCLUDED_CANDIDATE_ACCEPTED_PATH_ONLY", "BLOCKED_UNKNOWN"}
if any(item.get("candidate_disposition") not in allowed_omitted_dispositions or item.get("normative") is not False or not item.get("candidate_basis") or not item.get("minimum_evidence") for item in omitted):
    raise SystemExit("omitted fields need explicit non-normative disposition, basis and closure evidence")
for item in omitted:
    if item["candidate_disposition"] == "BLOCKED_UNKNOWN" and not any(key.startswith("blocks_") and value is True for key, value in item.items()):
        raise SystemExit("blocked unknown omitted field lacks a dependent blocker: " + item["omitted_id"])
    if item["candidate_disposition"].startswith("EXCLUDED_CANDIDATE") and item.get("profile_scope") not in ("selected run only", "selected profile only", "accepted selected run only"):
        raise SystemExit("excluded candidate lacks a narrow scope: " + item["omitted_id"])
if inventory["canonicalization"]["field_order"] != "NOT_DEFINED" or inventory["canonicalization"]["digest_preimage"] != "NOT_DELIVERED" or inventory["canonicalization"]["cold_vector"] != "NOT_GENERATED":
    raise SystemExit("field inventory incorrectly claims a canonical encoding or vector")

root_audit = load_json(FIXTURE / "PARENT-COLD-ROOT-PARTICIPATION-AUDIT.json")
independent_check = load_json(FIXTURE / "INDEPENDENT-CHECK.json")
if len(independent_check["checks"]) != 52 or not all(check.get("pass") is True for check in independent_check["checks"]):
    raise SystemExit("v35 independent research check is not 52/52 PASS")
if (
    root_audit["admission_conclusion"]["native_runtime_root_understood_for_selected_scope"] is not True
    or root_audit["admission_conclusion"]["remaining_required_reverse_engineering"] != 0
    or root_audit["admission_conclusion"]["gate1_closed"] is not False
    or root_audit["result"]["product_wide_absence_claim"] is not False
):
    raise SystemExit("v35 root audit does not support the bounded selected-scope conclusion")

admission_review = next(
    (item for item in inventory["evidence_overlays"] if item.get("overlay_id") == "v35_selected_parent_admission_review"),
    None,
)
if admission_review is None:
    raise SystemExit("v35 parent admission review overlay missing")
if (
    admission_review["package_identity"] != implementer["identity"]
    or admission_review["research_manifest_sha256"] != implementer["source_evidence"]["research_manifest_sha256"]
    or admission_review["research_independent_check_sha256"] != implementer["source_evidence"]["research_independent_check_sha256"]
    or admission_review["independent_check_count"] != 52
    or admission_review["independent_check_all_pass"] is not True
    or admission_review["selected_gate1_reverse_dependencies"] != 0
    or admission_review["gate1_closed"] is not False
    or admission_review["remaining_clean_room_design"] != [
        "approved canonical framing and digest domain",
        "field projection and optional-state choices",
        "typed constructor/root receipt schema",
    ]
    or admission_review["remaining_clean_room_validation"] != [
        "complete cold vector",
        "independent parse-reserialize equality",
        "success/failure/fresh-reconstruction receipt vectors",
    ]
):
    raise SystemExit("v35 parent admission review overlay mismatch")

print(f"parent cold field inventory checks passed ({len(entries)} entries; {len(omitted)} omitted blockers; {len(registered_sources)} inventory sources; 44/44 package assets verified; v35 Gate1 dependencies=0; readiness remains NO)")
