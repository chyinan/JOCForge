# Imperative Shell: read hash-pinned washed fixtures and emit a non-normative inventory file.
"""Build a traceable, explicitly non-normative field inventory from washed v35 assets."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE_DIR = ROOT / "crates/jocforge-domain/tests/fixtures/spatial_control_cold_serializer_blocker"
PACKAGE_MANIFEST = FIXTURE_DIR / "SPATIAL-CONTROL-COLD-SERIALIZER-BLOCKER-MANIFEST.json"
EXPECTED_PACKAGE_SHA256 = "71ca9bb30c2129019a76ecb703df47a88f5384582bb65a53faee92103e9dcd0b"
OUTPUT = ROOT / "docs/implementation/PARENT-COLD-FIELD-INVENTORY.json"
OUTPUT_MANIFEST = ROOT / "docs/implementation/PARENT-COLD-FIELD-INVENTORY-MANIFEST.json"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


manifest_bytes = PACKAGE_MANIFEST.read_bytes()
if digest(manifest_bytes) != EXPECTED_PACKAGE_SHA256:
    raise SystemExit("the sanitized implementer fixture is not v35")
package_manifest = json.loads(manifest_bytes.decode("utf-8"), object_pairs_hook=unique_pairs)
if package_manifest.get("identity") != "spatial-control-cold-serializer.blocked.v35":
    raise SystemExit("unexpected washed package identity")
if len(package_manifest.get("files", [])) != 44:
    raise SystemExit("v35 package asset count mismatch")
declared_files = {entry["file"]: entry for entry in package_manifest["files"]}
asset_cache = {}
source_registry = {}


def asset(name: str):
    if name not in declared_files:
        raise ValueError(f"source is not declared by v35 manifest: {name}")
    if name not in asset_cache:
        data = (FIXTURE_DIR / name).read_bytes()
        declaration = declared_files[name]
        if len(data) != declaration["bytes"] or digest(data) != declaration["sha256"]:
            raise ValueError(f"source asset hash mismatch: {name}")
        asset_cache[name] = json.loads(data.decode("utf-8"), object_pairs_hook=unique_pairs)
        source_registry[name] = {"sha256": digest(data), "bytes": len(data)}
    return asset_cache[name]


def json_pointer(document, pointer: str):
    value = document
    if pointer:
        if not pointer.startswith("/"):
            raise ValueError(f"invalid JSON pointer: {pointer}")
        for part in pointer[1:].split("/"):
            part = part.replace("~1", "/").replace("~0", "~")
            value = value[int(part)] if isinstance(value, list) else value[part]
    return value


def selected_value(record):
    if "submitted_value" in record:
        return {"observed": record.get("observed", "NOT_CAPTURED"), "submitted": record["submitted_value"]}
    for key in ("observed", "selected_value", "value"):
        if key in record:
            return record[key]
    return "NOT_CAPTURED"


def field_entry(
    inventory_id: str,
    group: str,
    source_file: str,
    pointer: str,
    location: str,
    type_width: str,
    semantic: str,
    stage: dict,
    candidate_disposition: str,
    blocker: str,
    profile_scope: str = "selected ADM BWF / music_rectangular_native_109",
):
    record = json_pointer(asset(source_file), pointer)
    return {
        "inventory_id": inventory_id,
        "group": group,
        "source": {
            "asset": source_file,
            "sha256": source_registry[source_file]["sha256"],
            "json_pointer": pointer,
        },
        "native_observation": {
            "physical_location": location,
            "type_width": type_width,
            "selected_value": selected_value(record),
            "semantic": semantic,
            "evidence_status": record.get("status", "NATIVE_OBSERVED_BOUNDED"),
            "producer_stage": stage["producer"],
            "consumer_stage": stage["consumer"],
            "source_record": record,
        },
        "clean_room_candidate": {
            "disposition": candidate_disposition,
            "normative": False,
            "basis": f"sanitized source record {source_file}{pointer}",
            "profile_scope": profile_scope,
        },
        "blocker": blocker,
    }


entries = []
source_counts = {}

admission_matrix = asset("PARENT-ADMISSION-EVIDENCE-MATRIX.json")
root_participation = asset("PARENT-COLD-ROOT-PARTICIPATION-AUDIT.json")
lifecycle_contract = asset("PARENT-LIFECYCLE-CONTRACT.json")
block_inventory = asset("SELECTED-PARENT-COLD-BLOCK-INVENTORY.json")
independent_check = asset("INDEPENDENT-CHECK.json")
admission = package_manifest["parent_admission"]
if (
    admission["selected_gate1_reverse_dependencies"] != 0
    or admission["gate1_closed"] is not False
    or admission["m1_canonical_schema_and_digest"] != "CLEAN_ROOM_DESIGN_REQUIRED"
    or admission["m2_serializer_parser_cold_roundtrip"] != "NOT_RUN"
    or admission["m3_receipt_transaction_vectors"] != "NOT_RUN"
):
    raise ValueError("v35 parent admission state differs from the washed handoff")
if (
    len(independent_check["checks"]) != 52
    or not all(check.get("pass") is True for check in independent_check["checks"])
):
    raise ValueError("v35 independent research check is not 52/52 PASS")
if (
    root_participation["admission_conclusion"]["remaining_required_reverse_engineering"] != 0
    or root_participation["admission_conclusion"]["gate1_closed"] is not False
    or root_participation["result"]["product_wide_absence_claim"] is not False
):
    raise ValueError("v35 root-participation audit does not support the bounded admission conclusion")
if block_inventory["remaining"]["reverse_engineering_required_selected_scope"] != 0:
    raise ValueError("v35 block inventory still lists selected-scope reverse-engineering work")


def add_records(group, prefix, source_file, array_pointer, location_prefix, stage, classify):
    records = json_pointer(asset(source_file), array_pointer)
    if not isinstance(records, list):
        raise ValueError(f"source array missing: {source_file}{array_pointer}")
    source_counts[group] = len(records)
    for index, record in enumerate(records):
        offset = record.get("offset", record.get("descriptor_offset", index))
        width = record.get("bytes", 4)
        location = f"{location_prefix}+{offset}" if width == 1 else f"{location_prefix}+{offset}..+{offset + width - 1}"
        type_width = f"{record.get('type', 'NOT_DEFINED')} / {width} byte(s)"
        semantic = record.get("semantic", record.get("name", "NOT_CAPTURED"))
        disposition, blocker = classify(record)
        pointer = f"{array_pointer}/{index}"
        entries.append(
            field_entry(
                f"{prefix}.offset_{offset:03d}",
                group,
                source_file,
                pointer,
                location,
                type_width,
                semantic,
                stage,
                disposition,
                blocker,
            )
        )


def classify_bound(record):
    status = str(record.get("status", ""))
    kind = str(record.get("type", ""))
    semantic = str(record.get("semantic", ""))
    if "pointer" in kind.lower():
        return "EXCLUDED_POINTER_CANDIDATE", "runtime pointer is not a portable field"
    if status == "OBSERVED_ZERO" or "padding/reserved" in semantic:
        return "ZEROED_SELECTED_RUN_ONLY", "zero is observed only in the declared bounded run"
    if "NOT_CAPTURED" in status or "PARTIAL" in status or "UNKNOWN" in status:
        return "BLOCKED_UNKNOWN", "field semantics or cross-profile behavior remain incomplete"
    if "NOT_CURRENT_ADAPTER_CONSUMED" in status:
        return "NOT_CONSUMED_BY_CURRENT_ADAPTER", "do not infer a canonical consumer from storage alone"
    if status.startswith("BOUND"):
        return "CONSUMED_CANDIDATE_NOT_NORMATIVE", "native mapping is bounded; portable encoding is not selected"
    return "NOT_DECIDED", "no normative serialization decision exists"


config_name = "PARENT-COLD-CONFIG-SEMANTICS.json"
config = asset(config_name)
source_counts["descriptor"] = len(config["descriptor_fields"])
for index, record in enumerate(config["descriptor_fields"]):
    offset = record["offset"]
    width = record["bytes"]
    location = f"descriptor+{offset}..+{offset + width - 1}"
    disposition, blocker = classify_bound(record)
    entries.append(field_entry(
        f"descriptor.offset_{offset:03d}", "descriptor", config_name, f"/descriptor_fields/{index}",
        location, f"{record['type']} / {width} byte(s)", record.get("semantic", "NOT_CAPTURED"),
        {"producer": "descriptor adapter 0x178dc0 (bounded)" if offset != 0 else "descriptor producer (bounded)", "consumer": "parent query/constructor scope"},
        disposition, blocker,
    ))
add_records("policy", "policy_block", config_name, "/policy_block_44", "policy", {"producer": "policy producer 0x4e14c0 (selected run)", "consumer": "property setter path (selected run)"}, classify_bound)
add_records("measurement_summary", "measurement_summary", config_name, "/measurement_summary_known_fields", "descriptor", {"producer": "measurement producer/adapter (bounded)", "consumer": "measurement export / dialnorm consumer (bounded)"}, classify_bound)

prefix_name = "SELECTED-CONSTRUCTOR-PREFIX-DYNAMIC-JOIN.json"
prefix = asset(prefix_name)
source_counts["selected_prefix"] = len(prefix["fields"])
for index, record in enumerate(prefix["fields"]):
    offset = record["offset"]
    kind = record["type"]
    semantic = record.get("semantic", "NOT_CLOSED")
    value = record.get("selected_value", record.get("value", "NOT_CAPTURED"))
    record_for_classification = {"type": kind, "semantic": semantic, "status": "NOT_CLOSED" if semantic in ("NOT_CLOSED", "Priv_or_reporter_pointer_low", "Priv_or_reporter_pointer_high") else "BOUND"}
    disposition, blocker = classify_bound(record_for_classification)
    if record.get("selected_branch_consumption") == "NOT_JOINED":
        disposition, blocker = "BLOCKED_BRANCH_CONSUMPTION_NOT_JOINED", "selected branch consumption has not been joined"
    if offset in (96, 100):
        disposition, blocker = "BLOCKED_NO_CONSUMER", "no scoped inner consumer was observed"
    entries.append({
        "inventory_id": f"selected_prefix.offset_{offset:03d}",
        "group": "selected_prefix",
        "source": {"asset": prefix_name, "sha256": source_registry[prefix_name]["sha256"], "json_pointer": f"/fields/{index}"},
        "native_observation": {"physical_location": f"prefix+{offset}..+{offset+3}", "type_width": f"{kind} / 4 byte(s)", "selected_value": value, "raw_word_hex": record.get("word_hex", "NOT_CAPTURED"), "semantic": semantic, "evidence_status": "NATIVE_OBSERVED_SELECTED_RUN", "producer_stage": "fresh inner construction path", "consumer_stage": "descriptor builder/parent constructor same generation", "source_record": record},
        "clean_room_candidate": {"disposition": disposition, "normative": False, "basis": f"sanitized source record {prefix_name}/fields/{index}", "profile_scope": "selected run only"},
        "blocker": blocker,
    })

external_structure_name = "EXTERNAL-METADATA-STRUCTURE-AND-REMAINING-FIELDS.json"
add_records("external_metadata_layout", "external_metadata", external_structure_name, "/external_metadata_120_structure", "packed", {"producer": "metadata producer 0x4e0890 (bounded)", "consumer": "packed copy 0x178dc0 (conditional)"}, classify_bound)

channel_name = "CHANNEL-EXTERNAL-METADATA-TRIM-SEMANTICS.json"
add_records("channel_record", "channel_record", channel_name, "/channel_record_6/fields", "channel_record", {"producer": "host channel-record assembly (bounded)", "consumer": "raw96-to-DTO adapter 0x426e80"}, classify_bound)

trim_name = "TRIM-PAYLOAD-PRODUCER-CONSUMER-EVIDENCE.json"
add_records("trim_payload", "trim_payload", trim_name, "/consumer/payload_fields", "trim_payload", {"producer": "trim producer/adapter (bounded)", "consumer": "trim consumer 0x1acfe0"}, classify_bound)

core_name = "CORE-MEASUREMENT-SLOT-CONSUMER-BOUNDARY.json"
core_slots = asset(core_name)["slots"]
source_counts["measurement_core_slots"] = len(core_slots)
for index, record in enumerate(core_slots):
    core_offset = record["core_offset"]
    public_offset = record["public_offset"]
    entries.append({
        "inventory_id": f"measurement_core.offset_{core_offset:03d}",
        "group": "measurement_core_slots",
        "source": {"asset": core_name, "sha256": source_registry[core_name]["sha256"], "json_pointer": f"/slots/{index}"},
        "native_observation": {"physical_location": f"core+{core_offset} -> public result+{public_offset}", "type_width": "F32 / 4 bytes, scale 128", "selected_value": "NOT_CAPTURED", "semantic": record["semantic_name"], "evidence_status": "OFFSET_AND_SCALE_CLOSED_NAME_NOT_CLOSED", "producer_stage": f"core writer 0x1b7d10 write {record['write_rva']}", "consumer_stage": f"getter {record['getter_rva']} and adapter 0x1ae790", "source_record": record},
        "clean_room_candidate": {"disposition": "BLOCKED_UNKNOWN_SEMANTIC", "normative": False, "basis": "field name and complete F32 canonicalization are not defined", "profile_scope": "bounded selected run"},
        "blocker": "public measurement semantic names remain NOT_CLOSED",
    })

dictionary_name = "EXTERNAL-METADATA-PRODUCER-COVERAGE-AND-SEPARATE-DICTIONARY.json"
dictionary = asset(dictionary_name)
names = dictionary["separate_ddp_metadata_dictionary"]["field_names_in_index_order"]
source_counts["ddp_dictionary"] = len(names)
for index, name in enumerate(names):
    pointer = f"/separate_ddp_metadata_dictionary/field_names_in_index_order/{index}"
    entries.append({
        "inventory_id": f"ddp_dictionary.word_{index:02d}",
        "group": "ddp_dictionary",
        "source": {"asset": dictionary_name, "sha256": source_registry[dictionary_name]["sha256"], "json_pointer": pointer},
        "native_observation": {"physical_location": f"separate dictionary word[{index}] (no packed offset mapping)", "type_width": "U32 / 4 byte(s)", "selected_value": "NOT_CAPTURED", "semantic": name, "evidence_status": "NATIVE_DICTIONARY_NAME_OBSERVED_IDENTITY_NOT_JOINED", "producer_stage": "dictionary serializer route (not invoked in accepted run)", "consumer_stage": "dictionary parser route (not invoked in accepted run)", "source_record": name},
        "clean_room_candidate": {"disposition": "BLOCKED_UNJOINED_LAYOUT", "normative": False, "basis": "dictionary is separate and has no same-owner/same-generation join to packed120", "profile_scope": "bounded current run"},
        "blocker": "offset transplantation is forbidden until the identity join is proven",
    })

framing_name = "REQUIRED-FRAMING-RULES.json"
add_records("parent_framing", "framing", framing_name, "/rules", "parent envelope", {"producer": "CLEAN_ROOM_DESIGN (not native observed)", "consumer": "future reference parser (not implemented)"}, lambda record: ("NOT_SELECTED", "framing rule remains blocked or conflict-blocked"))

host_name = "PARENT-COLD-HOST-LIFECYCLE-BOUNDARY.json"
host = asset(host_name)
source_counts["host_lifecycle"] = len(host["host_fields"])
for index, record in enumerate(host["host_fields"]):
    offset = record["offset"]
    entries.append({
        "inventory_id": f"host_lifecycle.offset_{offset:03d}",
        "group": "host_lifecycle",
        "source": {"asset": host_name, "sha256": source_registry[host_name]["sha256"], "json_pointer": f"/host_fields/{index}"},
        "native_observation": {"physical_location": f"host+{offset}", "type_width": "role-dependent / exact access widths in source record", "selected_value": "NOT_CAPTURED", "semantic": record["role"], "evidence_status": "NATIVE_LIFECYCLE_BOUNDARY_OBSERVED", "producer_stage": record.get("constructor", "NOT_CAPTURED"), "consumer_stage": record.get("open_commit", record.get("cleanup", "bounded lifecycle")), "source_record": record},
        "clean_room_candidate": {"disposition": "EXCLUDED_HOST_RUNTIME_STATE_CANDIDATE", "normative": False, "basis": "host lifecycle owner is not parent cold persistent state", "profile_scope": "bounded host lifecycle"},
        "blocker": "do not persist host flags or process pointers as parent canonical fields",
    })

for supporting_asset in (
    "PACKED-METADATA-OWNER-CONSTRUCTOR.json",
    "PACKED-METADATA-DYNAMIC-CURRENT-RUN.json",
    "PARENT-METADATA-CAUSAL-JOIN.json",
    "CROSS-PROFILE-ACTUAL-RUNS.json",
    "PARENT-METADATA-PROVIDER.json",
    "PARENT-COLD-FRESH-RECONSTRUCTION.json",
    "DTHD-METER-PIMPL-BOUNDARY.json",
    "TARGETED-LOUDNESS-TYPE-CROSS-MODULE-NO-HIT.json",
):
    asset(supporting_asset)

provider_name = "PARENT-METADATA-PROVIDER.json"
provider = asset(provider_name)
entries.append({
    "inventory_id": "parent_metadata_provider.object",
    "group": "parent_metadata_provider",
    "source": {"asset": provider_name, "sha256": source_registry[provider_name]["sha256"], "json_pointer": "/provider"},
    "native_observation": {"physical_location": "composite_master+264", "type_width": "120-byte metadata block in a 488-byte owner", "selected_value": "NOT_CAPTURED", "semantic": provider["provider"]["class"], "evidence_status": "NATIVE_PROVIDER_BOUND_SELECTED_SOURCE_NOT_CAPTURED", "producer_stage": "selected source virtual+0xc8 writer", "consumer_stage": "parent descriptor+56 consumer 0x178dc0", "source_record": provider["provider"]},
    "clean_room_candidate": {"disposition": "BLOCKED_SOURCE_MASTER_JOIN", "normative": False, "basis": "selected source class and same underlying master are not joined", "profile_scope": "selected current run"},
    "blocker": "composite owner+264 is not yet joined to producer owner+88 by selected source identity",
})

fresh_name = "PARENT-COLD-FRESH-RECONSTRUCTION.json"
fresh = asset(fresh_name)
entries.append({
    "inventory_id": "parent_lifecycle.fresh_inner",
    "group": "parent_lifecycle",
    "source": {"asset": fresh_name, "sha256": source_registry[fresh_name]["sha256"], "json_pointer": "/classification"},
    "native_observation": {"physical_location": "new EmpireEnc inner per record", "type_width": "runtime object, size 0x238 bytes", "selected_value": fresh["classification"], "semantic": "fresh construction/reconstruction; same-object restore false", "evidence_status": "NATIVE_FRESH_RECONSTRUCTION_SCOPED", "producer_stage": "run path 0x175500 constructs fresh 0x177e50 instance", "consumer_stage": "process/flush/close/destroy", "source_record": fresh["classification"]},
    "clean_room_candidate": {"disposition": "CANDIDATE_FRESH_RECONSTRUCTION_NOT_RESTORE", "normative": False, "basis": "bounded lifecycle observation only", "profile_scope": "scoped native run path"},
    "blocker": "dynamic second run and receipt/token retirement acceptance remain NOT_RUN",
})

dthd_name = "DTHD-METER-PIMPL-BOUNDARY.json"
dthd = asset(dthd_name)
entries.append({
    "inventory_id": "dthd_public_meter.layout",
    "group": "external_module_type_boundary",
    "source": {"asset": dthd_name, "sha256": source_registry[dthd_name]["sha256"], "json_pointer": "/meter"},
    "native_observation": {"physical_location": "DTHD public wrapper extent", "type_width": "16-byte public object: vptr at +0, PIMPL at +8", "selected_value": "public +12 has no distinct typed access; +16 is outside observed extent", "semantic": "DTHD wrapper layout, not DEE DDP field layout", "evidence_status": "NATIVE_RTTI_AND_WRAPPER_BOUNDARY", "producer_stage": "DTHD module constructor", "consumer_stage": "DTHD meter adapters/direct callers", "source_record": dthd["meter"]},
    "clean_room_candidate": {"disposition": "EXCLUDED_FROM_DEE_LAYOUT_MAPPING", "normative": False, "basis": "cross-module type/layout transplant is forbidden", "profile_scope": "adjacent DTHD module only"},
    "blocker": "public offsets 8/12/16 are not corroborated as DEE measurement fields",
})

targeted_name = "TARGETED-LOUDNESS-TYPE-CROSS-MODULE-NO-HIT.json"
targeted = asset(targeted_name)
entries.append({
    "inventory_id": "dthd_loudness_rtti.types",
    "group": "external_module_type_boundary",
    "source": {"asset": targeted_name, "sha256": source_registry[targeted_name]["sha256"], "json_pointer": "/result"},
    "native_observation": {"physical_location": "64 PE files and 164 adjacent text files; 228 candidates", "type_width": "RTTI/type-name evidence only", "selected_value": targeted["result"]["target_module_types"], "semantic": "wrapper families identified without public offset layout", "evidence_status": "BOUNDED_RTTI_NO_HIT_FOR_PUBLIC_OFFSETS", "producer_stage": "targeted cross-module inventory", "consumer_stage": "no named consumer corroborating public offsets 8/12/16", "source_record": targeted["result"]},
    "clean_room_candidate": {"disposition": "NO_OFFSET_TRANSPLANT", "normative": False, "basis": "RTTI family names do not establish field layout", "profile_scope": "bounded module inventory"},
    "blocker": "no public-offset semantic name was corroborated",
})
source_counts["parent_metadata_provider"] = 1
source_counts["parent_lifecycle"] = 1
source_counts["external_module_type_boundary"] = 2

receipt_gap_name = "RECEIPT-TRANSACTION-BLOCKED-GAP.json"
receipt_gap = json.loads((ROOT / "docs/implementation" / receipt_gap_name).read_text(encoding="utf-8"), object_pairs_hook=unique_pairs)
source_registry[receipt_gap_name] = {"sha256": digest((ROOT / "docs/implementation" / receipt_gap_name).read_bytes()), "bytes": (ROOT / "docs/implementation" / receipt_gap_name).stat().st_size}
source_counts["receipt_contract"] = len(receipt_gap["undefined_contract_items"])
for index, record in enumerate(receipt_gap["undefined_contract_items"]):
    entries.append({
        "inventory_id": f"receipt.{record['id']}",
        "group": "receipt_contract",
        "source": {"asset": receipt_gap_name, "sha256": source_registry[receipt_gap_name]["sha256"], "json_pointer": f"/undefined_contract_items/{index}"},
        "native_observation": {"physical_location": "NOT_APPLICABLE", "type_width": "NOT_DEFINED", "selected_value": "NOT_DEFINED", "semantic": record["id"], "evidence_status": "CLEAN_ROOM_GAP_ONLY_NATIVE_RECEIPT_NOT_OBSERVED", "producer_stage": "future transaction coordinator (not implemented)", "consumer_stage": "future receipt validator (not implemented)", "source_record": record},
        "clean_room_candidate": {"disposition": "BLOCKED_NOT_DEFINED", "normative": False, "basis": "washed receipt gap record", "profile_scope": "selected profile design only"},
        "blocker": record["unknowns"],
    })

omitted_ledger = [
    {"omitted_id": "descriptor.mode1_tuple_full_semantics", "source_asset": "PARENT-COLD-CONFIG-SEMANTICS.json", "source_sha256": source_registry["PARENT-COLD-CONFIG-SEMANTICS.json"]["sha256"], "status": "PARTIAL", "candidate_disposition": "EXCLUDED_CANDIDATE_SELECTED_MODE0_ONLY", "candidate_basis": "selected profile observes mode 0 and mode tuple zero; this does not generalize to mode 1", "normative": False, "profile_scope": "selected run only", "blocks_canonical_vector": False, "minimum_evidence": "mode-1 per-word source, consumer and type only if mode 1 enters scope"},
    {"omitted_id": "external_metadata_other_profile_gap_writes", "source_asset": "PACKED-METADATA-DYNAMIC-CURRENT-RUN.json", "source_sha256": source_registry["PACKED-METADATA-DYNAMIC-CURRENT-RUN.json"]["sha256"], "status": "NOT_RUN_OR_NOT_JOINED", "candidate_disposition": "EXCLUDED_CANDIDATE_OUTSIDE_SELECTED_PROFILE", "candidate_basis": "other profiles are outside the single selected profile; no cross-profile rule is inferred", "normative": False, "profile_scope": "selected run only", "blocks_canonical_vector": False, "minimum_evidence": "per-profile first-writer/owner/generation traces if profile scope expands"},
    {"omitted_id": "measurement_public_8_12_16_semantic_names", "source_asset": "CORE-MEASUREMENT-SLOT-CONSUMER-BOUNDARY.json", "source_sha256": source_registry["CORE-MEASUREMENT-SLOT-CONSUMER-BOUNDARY.json"]["sha256"], "status": "NOT_CLOSED", "candidate_disposition": "BLOCKED_UNKNOWN", "candidate_basis": "three present measurement slots lack semantic names", "normative": False, "profile_scope": "selected run", "blocks_complete_field_inventory": True, "blocks_canonical_vector": True, "minimum_evidence": "named public-field owner or consumer binding"},
    {"omitted_id": "packed120_dictionary_same_owner_generation_join", "source_asset": dictionary_name, "source_sha256": source_registry[dictionary_name]["sha256"], "additional_source_asset": "CROSS-PROFILE-ACTUAL-RUNS.json", "additional_source_sha256": source_registry["CROSS-PROFILE-ACTUAL-RUNS.json"]["sha256"], "status": "NO_HIT_NOT_INVOKED", "candidate_disposition": "EXCLUDED_CANDIDATE_ACCEPTED_PATH_ONLY", "candidate_basis": "accepted embedded-DBMD path invokes packed producer/parent consumer but does not invoke the separate dictionary serializer/parser", "normative": False, "profile_scope": "accepted selected run only", "blocks_canonical_vector": False, "minimum_evidence": "same-owner/same-generation identity join if a different route requires the dictionary"},
    {"omitted_id": "f01e_parent_embedding", "source_asset": "REQUIRED-FRAMING-RULES.json", "source_sha256": source_registry["REQUIRED-FRAMING-RULES.json"]["sha256"], "status": "MISSING", "candidate_disposition": "BLOCKED_UNKNOWN", "candidate_basis": "parent embedding is not specified", "normative": False, "profile_scope": "selected parent schema", "blocks_canonical_vector": True, "minimum_evidence": "presence/version/length/representation in parent envelope"},
    {"omitted_id": "optional_cache_current_flags_writer_set_framing", "source_asset": "REQUIRED-FRAMING-RULES.json", "source_sha256": source_registry["REQUIRED-FRAMING-RULES.json"]["sha256"], "status": "PARTIAL_OR_CONFLICT_BLOCKED", "candidate_disposition": "BLOCKED_UNKNOWN", "candidate_basis": "cold absence does not define optional or non-empty representations", "normative": False, "profile_scope": "selected parent schema", "blocks_canonical_vector": True, "minimum_evidence": "presence, grouping, order, length and non-empty encoding"},
    {"omitted_id": "receipt_transition_fields_and_vectors", "source_asset": receipt_gap_name, "source_sha256": source_registry[receipt_gap_name]["sha256"], "status": "NOT_DEFINED_NOT_RUN", "candidate_disposition": "BLOCKED_UNKNOWN", "candidate_basis": "no typed receipt/transaction contract or executable vectors", "normative": False, "profile_scope": "selected parent transaction", "blocks_canonical_vector": True, "blocks_transaction_acceptance": True, "minimum_evidence": "typed receipt/token contract and required success/failure vectors"},
]

inventory = {
    "schema": "jocforge-parent-cold-field-inventory.v1",
    "identity": "parent-cold-field-inventory.v35",
    "status": "INVENTORY_INCOMPLETE_NOT_NORMATIVE_SCHEMA",
    "implementation_ready": "NO",
    "production_allowed": False,
    "selected_profile_scope": {"run_identity": "music_rectangular_native_109", "cross_profile_generalization": False},
    "source_implementer_manifest": {"sha256": EXPECTED_PACKAGE_SHA256, "identity": package_manifest["identity"]},
    "source_assets": [{"file": name, **record} for name, record in sorted(source_registry.items())],
    "expected_group_counts_from_sources": source_counts,
    "field_entries": entries,
    "evidence_overlays": [
        {"overlay_id": "packed_gap_selected_init_and_parent_final", "sources": ["PACKED-METADATA-OWNER-CONSTRUCTOR.json", "PACKED-METADATA-DYNAMIC-CURRENT-RUN.json", "PARENT-METADATA-CAUSAL-JOIN.json"], "status": "SELECTED_RUN_ONLY", "normative": False, "summary": "14 gap bytes selected initialization cold-zero; 80..87 writer/parent final-zero facts are scoped to observed generation"},
        {"overlay_id": "cross_profile_routes", "sources": ["CROSS-PROFILE-ACTUAL-RUNS.json"], "status": "ROUTE_SCOPED", "normative": False, "summary": "accepted embedded DBMD, stripped-sidecar rejection, and legal convert-to-ADM route are separate runs"},
        {
            "overlay_id": "v35_selected_parent_admission_review",
            "sources": [
                "PARENT-ADMISSION-EVIDENCE-MATRIX.json",
                "PARENT-COLD-ROOT-PARTICIPATION-AUDIT.json",
                "PARENT-LIFECYCLE-CONTRACT.json",
                "SELECTED-PARENT-COLD-BLOCK-INVENTORY.json",
                "INDEPENDENT-CHECK.json",
            ],
            "status": "SELECTED_SCOPE_REVERSE_ENGINEERING_CLOSED_DESIGN_AND_VALIDATION_BLOCKED",
            "normative": False,
            "package_identity": package_manifest["identity"],
            "research_manifest_sha256": package_manifest["source_evidence"]["research_manifest_sha256"],
            "research_independent_check_sha256": package_manifest["source_evidence"]["research_independent_check_sha256"],
            "independent_check_count": len(independent_check["checks"]),
            "independent_check_all_pass": True,
            "selected_gate1_reverse_dependencies": admission["selected_gate1_reverse_dependencies"],
            "gate1_closed": admission["gate1_closed"],
            "native_runtime_root_understood_for_selected_scope": root_participation["admission_conclusion"]["native_runtime_root_understood_for_selected_scope"],
            "native_canonical_root_participation_observed": root_participation["admission_conclusion"]["native_canonical_root_participation_observed"],
            "remaining_clean_room_design": block_inventory["remaining"]["clean_room_design"],
            "remaining_clean_room_validation": block_inventory["remaining"]["clean_room_validation"],
            "inherited_asset_identity_note": "The v35 package carries unchanged selected-run evidence assets whose internal source identities remain v34; package identity and admission review are v35.",
        },
    ],
    "omitted_field_ledger": omitted_ledger,
    "canonicalization": {"normative": False, "field_order": "NOT_DEFINED", "domain_separator": "NOT_DEFINED", "digest_preimage": "NOT_DELIVERED", "cold_vector": "NOT_GENERATED"},
    "expected_used_as_seed": False,
}

output_bytes = (json.dumps(inventory, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
OUTPUT.write_bytes(output_bytes)
inventory_manifest = {
    "schema": "jocforge-parent-cold-field-inventory-manifest.v1",
    "identity": inventory["identity"],
    "status": inventory["status"],
    "source_implementer_manifest_sha256": EXPECTED_PACKAGE_SHA256,
    "inventory_file": OUTPUT.name,
    "inventory_bytes": len(output_bytes),
    "inventory_sha256": digest(output_bytes),
    "source_asset_count": len(source_registry),
    "field_entry_count": len(entries),
    "omitted_field_count": len(omitted_ledger),
    "implementation_ready": "NO",
    "production_allowed": False,
}
OUTPUT_MANIFEST.write_text(json.dumps(inventory_manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(f"wrote {OUTPUT.relative_to(ROOT)}: {len(entries)} entries, {len(omitted_ledger)} omitted fields, {len(source_registry)} hashed sources; blocked")
