"""Independently validate the machine-readable, non-normative v34 field inventory."""
import hashlib
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "crates/jocforge-domain/tests/fixtures/spatial_control_cold_serializer_blocker"
IMPLEMENTER_MANIFEST = FIXTURE / "SPATIAL-CONTROL-COLD-SERIALIZER-BLOCKER-MANIFEST.json"
IMPLEMENTER_MANIFEST_SHA256 = "43f7fcf730c7c5c321aa884d1810aa039427d0ae2f97f7cc94f93bfc525d404b"
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
    raise SystemExit("v34 implementer manifest hash mismatch")
implementer = json.loads(implementer_bytes.decode("utf-8"), object_pairs_hook=unique_pairs)
if implementer.get("identity") != "spatial-control-cold-serializer.blocked.v34":
    raise SystemExit("unexpected implementer package identity")
package_files = {item["file"]: item for item in implementer["files"]}

inventory_manifest = load_json(INVENTORY_MANIFEST_PATH)
inventory_bytes = INVENTORY_PATH.read_bytes()
if len(inventory_bytes) != inventory_manifest["inventory_bytes"] or digest(inventory_bytes) != inventory_manifest["inventory_sha256"]:
    raise SystemExit("field inventory artifact hash mismatch")
inventory = json.loads(inventory_bytes.decode("utf-8"), object_pairs_hook=unique_pairs)

if inventory["source_implementer_manifest"]["sha256"] != IMPLEMENTER_MANIFEST_SHA256:
    raise SystemExit("field inventory points at a different implementer package")
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
            raise SystemExit(f"v34 manifest disagrees with source registry: {name}")

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
if not all(any(key.startswith("blocks_") and value is True for key, value in item.items()) for item in omitted):
    raise SystemExit("each omitted field must identify a blocked dependent contract")
if inventory["canonicalization"]["field_order"] != "NOT_DEFINED" or inventory["canonicalization"]["digest_preimage"] != "NOT_DELIVERED" or inventory["canonicalization"]["cold_vector"] != "NOT_GENERATED":
    raise SystemExit("field inventory incorrectly claims a canonical encoding or vector")

print(f"parent cold field inventory checks passed ({len(entries)} entries; {len(omitted)} omitted blockers; {len(registered_sources)} hashed sources)")
