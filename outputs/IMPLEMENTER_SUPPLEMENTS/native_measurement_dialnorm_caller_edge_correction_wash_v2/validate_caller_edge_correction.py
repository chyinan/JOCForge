"""Validate the sanitized caller-edge correction package."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"expected JSON object: {path.name}")
    return value


def main() -> None:
    root = Path(__file__).resolve().parent
    manifest = read_json(root / "MANIFEST.json")
    evidence = read_json(root / "EVIDENCE.json")

    require(manifest.get("schema") == "wash-evidence-manifest.v1", "manifest schema mismatch")
    require(manifest.get("status") == "FOR_WASH_REVIEW", "status must remain FOR_WASH_REVIEW")
    require(manifest.get("package_id") == "native-measurement-dialnorm-caller-edge-correction-wash-v2", "package id mismatch")
    require(manifest.get("recipient_role") == "wash-agent", "recipient must remain Wash")
    require(manifest.get("implementation_agent_contacted") is False, "implementation contact must remain false")
    for key in ("contains_raw_addresses", "contains_binary_or_decompilation_excerpt", "contains_pcm_or_bitstream", "contains_private_auth_or_key", "contains_local_absolute_paths"):
        require(manifest.get(key) is False, f"sanitization flag changed: {key}")

    listed: set[str] = set()
    for entry in manifest.get("files", []):
        name = entry["name"]
        listed.add(name)
        path = root / name
        require(path.is_file(), f"missing package file: {name}")
        require(sha256(path) == entry["sha256"], f"SHA-256 mismatch: {name}")
        require(path.stat().st_size == entry["bytes"], f"byte count mismatch: {name}")
        require(path.suffix.lower() not in {".wav", ".bw64", ".pcm", ".ec3", ".dll", ".exe", ".xml"}, f"forbidden payload/source type: {name}")
    expected = {"REPORT.md", "EVIDENCE.json", "validate_caller_edge_correction.py"}
    require(listed == expected, "package file set changed")

    for path in [root / "MANIFEST.json", *(root / name for name in listed)]:
        content = path.read_text(encoding="utf-8")
        require(not re.search(r"(?i)\b[A-Z]:[\\/]", content), f"local absolute path found: {path.name}")
        require(not re.search(r"(?i)0x[0-9a-f]{6,}", content), f"raw address found: {path.name}")
        require("DECOMPILED" + "_BEGIN" not in content, f"decompilation excerpt found: {path.name}")

    hit = evidence.get("runtime_hit", {})
    require(hit.get("helper_argument_shape") == ["field_pointer", "element_count", "width_bits"], "helper argument shape changed")
    require(hit.get("element_count") == 1, "helper element count changed")
    require(hit.get("width_bits") == 5, "helper width changed")
    require(hit.get("serializer_input_member_offset_bytes") == 8, "serializer member offset changed")
    require(hit.get("field_scalar_value") == "NOT_CAPTURED", "field scalar must remain uncaptured")
    require("generic_writer_scalar_u16" not in hit, "deprecated scalar-value interpretation present")
    require(hit.get("static_candidate_callsite_match") is False, "runtime hit must remain distinct from static candidate")
    require(hit.get("raw_return_address_or_pointer_persisted") is False, "raw addresses/pointers must not be persisted")
    require(evidence.get("ghidra_static_path", {}).get("field_semantic_tag") == "UNBOUND", "field tag must remain unbound")
    require(evidence.get("ghidra_static_path", {}).get("typed_field_producer") == "UNBOUND", "field producer must remain unbound")
    require(evidence.get("measurement_chain", {}).get("measurement_to_f06nd_drc") == "UNBOUND", "measurement/DRC join must remain unbound")
    require(evidence.get("measurement_chain", {}).get("f06nd_drc_to_f12_bsi_dialnorm") == "UNBOUND", "DRC/BSI join must remain unbound")
    require(evidence.get("measurement_chain", {}).get("playback_and_full_e2e") == "NOT_RUN", "playback must remain NOT_RUN")

    serialized = json.dumps(evidence, sort_keys=True)
    require(('"field_scalar_value": ' + str(1)) not in serialized, "field scalar must not be mislabeled as one")
    require(("U16=" + str(1)) not in serialized, "deprecated U16 value claim present")
    require(evidence.get("implementation_agent_contacted") is False, "implementation contact must remain false")

    print("package_files=HASH_AND_SIZE_MATCH")
    print("correction=ELEMENT_COUNT_1; FIELD_SCALAR_NOT_CAPTURED; WIDTH_5")
    print("sanitization=NO_LOCAL_PATHS_ADDRESSES_DECOMP_BINARY_AUDIO_OR_AUTH")
    print("measurement_dialnorm_join=UNBOUND; PLAYBACK=NOT_RUN")


if __name__ == "__main__":
    main()
