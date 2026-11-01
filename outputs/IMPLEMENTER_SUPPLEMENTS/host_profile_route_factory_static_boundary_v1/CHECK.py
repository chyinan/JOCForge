from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
require(manifest.get("schema") == "wash-only-research-manifest.v1", "manifest schema mismatch")
require(manifest.get("package_id") == "HOST-PROFILE-ROUTE-FACTORY-STATIC-BOUNDARY-V1", "package identity mismatch")
for payload in manifest.get("payloads", []):
    data = (ROOT / payload["file"]).read_bytes()
    require(len(data) == payload["bytes"], f"length mismatch: {payload['file']}")
    require(hashlib.sha256(data).hexdigest() == payload["sha256"], f"hash mismatch: {payload['file']}")
require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip() == hashlib.sha256(manifest_path.read_bytes()).hexdigest() + "  MANIFEST.json", "manifest sidecar mismatch")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(evidence["result"] == "PROFILE_FIELD_REGISTRATION_AND_PLUGIN_SETUP_OBSERVED; R138_PROFILE_TO_SELECTED_FACTORY_UNBOUND", "result status mismatch")
require(evidence["source_binding"]["route_key"] == "encode_to_atmos_ddp", "route mismatch")
require(evidence["source_binding"]["input_type"] == "audio.adm", "input mismatch")
require(evidence["source_binding"]["time_base"] == "embedded_timecode", "R138 timebase mismatch")
require(evidence["open_edges"]["route_profile_to_actual_factory_dispatch"] == "UNBOUND", "factory edge overclosed")
require(evidence["open_edges"]["route_conditioned_host_dispatch_callsite"] == "NOT_IDENTIFIED", "dispatch callsite overclaimed")
require(evidence["open_edges"]["parent_task_generation"] == "NOT_OBSERVED", "generation overclaimed")
require(evidence["scope"]["raw_decompiler_text_in_package"] is False, "raw decompiler scope drift")
require(evidence["scope"]["raw_function_symbols_in_package"] is False, "raw symbol scope drift")
require(evidence["scope"]["pcm_or_bitstream_read_or_saved"] is False, "payload scope drift")

vectors = json.loads((ROOT / "CONTROLLED-VECTORS.json").read_text(encoding="utf-8"))
expected = {
    "registered_time_base_without_consumer_join": "FIELD_REGISTERED; ROUTE_TO_FACTORY_UNBOUND",
    "generic_category_registration": "REGISTRATION_ONLY; NON_GATING_FOR_R138",
    "schema_builder_path": "SCHEMA_ONLY; FACTORY_SELECTION_UNBOUND",
    "route_name_equals_factory_descriptor": "VALUE_MATCH_ONLY; UNBOUND",
    "task_generation_missing": "NOT_OBSERVED",
}
actual = {case["id"]: case["expected"] for case in vectors["cases"]}
require(actual == expected, "controlled vector status mismatch")
require(vectors["native_observations"] == 0, "unexpected native observation")

for name in ("REPORT.md", "EVIDENCE.json", "CONTROLLED-VECTORS.json", "SOURCE-IDENTITIES.json"):
    text = (ROOT / name).read_text(encoding="utf-8")
    require("FUN_" not in text, f"raw function symbol leaked into {name}")

sources = json.loads((ROOT / "SOURCE-IDENTITIES.json").read_text(encoding="utf-8"))
require(sources["analysis"]["tool"] == "Ghidra" and sources["analysis"]["read_only_project"] is True, "analysis provenance mismatch")
print("PASS: sanitized package hashes, field/setup roles, non-gating vectors, and UNBOUND route/factory edge")
