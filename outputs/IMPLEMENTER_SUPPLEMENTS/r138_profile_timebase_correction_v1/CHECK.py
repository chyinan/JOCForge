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
require(manifest.get("package_id") == "R138-PROFILE-TIMEBASE-CORRECTION-V1", "package identity mismatch")
for payload in manifest.get("payloads", []):
    data = (ROOT / payload["file"]).read_bytes()
    require(len(data) == payload["bytes"], f"length mismatch: {payload['file']}")
    require(hashlib.sha256(data).hexdigest() == payload["sha256"], f"hash mismatch: {payload['file']}")
require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip() == hashlib.sha256(manifest_path.read_bytes()).hexdigest() + "  MANIFEST.json", "manifest sidecar mismatch")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(evidence["result"] == "R138_TIME_BASE_CORRECTED; PRIOR_FILE_POSITION_CLAIM_RETRACTED; ROUTE_TO_FACTORY_UNBOUND", "result status drift")
require(evidence["corrected_source"]["execution_config_sha256"] == "fa65143f0bd7de52b36367137e61d681baa4c17824aded10be26317e7d1018a4", "source hash drift")
require(evidence["corrected_source"]["time_base"] == "embedded_timecode", "corrected field mismatch")
require(evidence["supersedes"]["prior_value"] == "file_position", "prior claim not identified")
require(evidence["supersedes"]["prior_claim_status"] == "RETRACTED_DO_NOT_USE_FOR_R138", "prior claim not retracted")
require(evidence["unaffected_edges"]["profile_owner_to_selected_factory_index"] == "UNBOUND", "factory edge overclosed")
require(evidence["unaffected_edges"]["parent_task_generation"] == "NOT_OBSERVED", "generation overclaimed")

vectors = json.loads((ROOT / "CONTROLLED-VECTORS.json").read_text(encoding="utf-8"))
actual = {case["id"]: case["expected"] for case in vectors["cases"]}
require(actual["hash_bound_r138_timebase"] == "embedded_timecode", "timebase vector mismatch")
require(actual["prior_package_value"] == "RETRACTED_DO_NOT_USE_FOR_R138", "retraction vector mismatch")
require(actual["route_and_input_unchanged"] == "HASH_BOUND_R138_ROUTE_AND_INPUT", "route/input vector mismatch")
require(actual["factory_selection"] == "UNBOUND", "factory edge vector mismatch")
require(evidence["scope"]["pcm_or_bitstream_read_or_saved"] is False, "payload scope drift")
require(evidence["scope"]["key_license_or_authentication_accessed"] is False, "scope drift")

sources = json.loads((ROOT / "SOURCE-IDENTITIES.json").read_text(encoding="utf-8"))
require(sources["analysis"]["tool"] == "Ghidra" and sources["analysis"]["read_only_project"] is True, "analysis provenance mismatch")
print("PASS: correction package hashes, R138 timebase retraction, and open route/factory edges")
