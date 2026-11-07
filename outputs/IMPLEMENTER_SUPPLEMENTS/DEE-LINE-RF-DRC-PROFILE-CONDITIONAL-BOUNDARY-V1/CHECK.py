from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LINE-RF-DRC-PROFILE-CONDITIONAL-BOUNDARY-V1"


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
payloads = {entry["file"]: entry for entry in manifest["payloads"]}
require(set(payloads) == expected, "payload set mismatch")
for name, entry in payloads.items():
    path = ROOT / name
    require(path.is_file(), f"missing payload: {name}")
    require(path.stat().st_size == entry["bytes"], f"payload size mismatch: {name}")
    require(sha256(path) == entry["sha256"], f"payload hash mismatch: {name}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(evidence["implementation_ready"] is False, "must remain reference-only")
require(len(evidence["line"]["tokens"]) == 6, "Line profile map incomplete")
require(evidence["line"]["xml_to_selected_plugin_handoff"] == "UNBOUND", "Line handoff boundary changed")
require(evidence["line"]["serializer_activation"] == "UNOBSERVED", "Line serializer gate changed")
require(evidence["line"]["decoded_pcm_effect"] == "NOT_DETERMINED_STATICALLY", "Line audio effect overclaimed")
require(evidence["rf"]["runtime_value"] == "UNBOUND", "RF runtime value must remain unbound")
require(evidence["rf"]["audio_or_bsi_effect"] == "UNBOUND/NOT_RUN", "RF effect must remain open")
require(evidence["execution"]["dee_run"] == "NOT_RUN", "DEE must remain NOT_RUN")
vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
require(vectors["status"] == "INPUTS_ONLY; NO_DERIVED_OUTPUTS; NOT_OBSERVED", "input-only vector status mismatch")
print(f"PASS mode={'optimized' if not __debug__ else 'normal'} line_tokens=6 rf=UNBOUND dee=NOT_RUN")
