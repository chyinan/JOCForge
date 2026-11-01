from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


manifest_raw = (ROOT / "MANIFEST.json").read_bytes()
manifest_hash = hashlib.sha256(manifest_raw).hexdigest()
expected_hash = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(manifest_hash == expected_hash, "manifest hash mismatch")
manifest = json.loads(manifest_raw)
require(manifest.get("schema") == "washed-pcm24-s16-f32-transfer-boundary.v1", "schema mismatch")
require(manifest.get("status") == "STATIC_DIRECT_CONSUMER_SLICE_NEGATIVE_F32_JOIN_OPEN", "status mismatch")

for entry in manifest.get("payloads", []):
    name = entry.get("file")
    require(isinstance(name, str) and name, "payload name missing")
    path = (ROOT / name).resolve()
    require(path.is_relative_to(ROOT.resolve()), "payload escapes package")
    require(path.is_file(), f"payload missing: {name}")
    data = path.read_bytes()
    require(len(data) == entry.get("bytes"), f"payload size mismatch: {name}")
    require(hashlib.sha256(data).hexdigest() == entry.get("sha256"), f"payload hash mismatch: {name}")

obs = json.loads((ROOT / "OBSERVATION.json").read_text(encoding="utf-8"))
require(obs.get("implementation_disposition", {}).get("production_rule_approved") is False, "production rule must remain unapproved")
require(obs.get("scope", {}).get("native_execution") == "NOT_RUN", "native status changed")
require(obs.get("direct_consumer_trace", {}).get("typed_row_join") == "UNBOUND", "row join scope mismatch")
require(obs.get("next_minimum_experiment", {}).get("status") == "PLAN_ONLY_WAITING_FOR_RUN144", "experiment gate mismatch")

for name in ("WASH-REVIEW.md", "OBSERVATION.json", "VERIFICATION.json"):
    text = (ROOT / name).read_text(encoding="utf-8")
    require("FUN_180" not in text and "0x" not in text, "reverse-engineering coordinates leaked")
    require("D:\\" not in text and "C:\\" not in text, "absolute path leaked")

print("WASHED_PCM24_F32_BOUNDARY=PASS")
print("PCM24_TO_F32_RULE=UNBOUND")
