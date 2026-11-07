from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-METERING-MODE-STATIC-CALLER-BOUNDARY-V2"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
manifest_sha = hashlib.sha256(manifest_bytes).hexdigest()
require(manifest["package_id"] == PACKAGE_ID, "manifest package id mismatch")
require(seal["package_id"] == PACKAGE_ID, "seal package id mismatch")
require(seal["manifest_sha256"] == manifest_sha, "seal manifest binding mismatch")
expected = {"REPORT.md", "EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "SOURCE-MANIFESTS.json", "CHECK.py"}
payloads = {item["file"]: item for item in manifest["payloads"]}
require(set(payloads) == expected, "payload set mismatch")
for name, item in payloads.items():
    path = ROOT / name
    require(path.is_file(), f"missing payload: {name}")
    require(path.stat().st_size == item["bytes"], f"payload size mismatch: {name}")
    require(sha256(path) == item["sha256"], f"payload hash mismatch: {name}")
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(evidence["implementation_ready"] is False, "must remain reference-only")
require(evidence["xml_property"]["selected_provider_join"] == "UNBOUND", "provider join must remain unbound")
require(evidence["processing_caller"]["processing_this_to_candidate_consumer_receiver"] == "BOUND_STATICALLY", "processing caller join mismatch")
require(evidence["processing_caller"]["xml_schema_owner_to_processing_this_or_receiver"] == "UNBOUND", "XML owner join must remain unbound")
require(evidence["execution"]["dee_run"] == "NOT_RUN", "DEE must remain not run")
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
require(vectors["status"] == "INPUTS_ONLY; NO_DERIVED_OUTPUTS; NOT_OBSERVED", "vector status mismatch")
print(f"PASS mode={'optimized' if not __debug__ else 'normal'} caller=BOUND XML_owner=UNBOUND effect=NOT_RUN")
