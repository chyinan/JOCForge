from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


raw = (ROOT / "MANIFEST.json").read_bytes()
expected = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(raw).hexdigest() == expected, "manifest hash mismatch")
manifest = json.loads(raw)
require(manifest.get("schema") == "washed-native-measurement-field-owner-observation.v1", "schema mismatch")
require(manifest.get("status") == "SAME_OWNER_SERIALIZER_CONSUMPTION_OBSERVED; F06_PRODUCER_JOIN_UNBOUND", "status mismatch")

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
require(obs.get("same_owner_dynamic_observation", {}).get("same_owner_join") is True, "owner join not recorded")
require(obs.get("same_owner_dynamic_observation", {}).get("F06_measurement_to_owner_field_join") == "UNBOUND", "F06 gap must remain explicit")
require(obs.get("implementation_disposition", {}).get("production_formula_approved") is False, "candidate formula must not be approved")
for name in ("WASH-REVIEW.md", "OBSERVATION.json", "VERIFICATION.json"):
    text = (ROOT / name).read_text(encoding="utf-8")
    require("0x1" not in text and "FUN_180" not in text, "reverse-engineering coordinates leaked")
    require("D:\\" not in text and "C:\\" not in text, "absolute path leaked")

print("WASHED_RUN144_OWNER_EDGE=PASS")
print("F06_MEASUREMENT_FORMULA=UNBOUND")
