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
require(manifest.get("schema") == "washed-adm-bwf-first-sample-range-crosscheck.v1", "schema mismatch")
require(manifest.get("status") == "CONDITIONAL_RANGE_FORMULA_STATIC; RESOLVERS_AND_FIRST_READER_UNBOUND", "status mismatch")

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
require(obs.get("scope", {}).get("native_job") == "NOT_RUN", "native status mismatch")
require(obs.get("metadata_crosscheck", {}).get("channels") == 36, "input profile scope mismatch")
require(obs.get("full_file_default_boundary", {}).get("actual_first_reader_request") == "UNBOUND", "reader boundary must stay explicit")

for name in ("WASH-REVIEW.md", "OBSERVATION.json", "VERIFICATION.json"):
    text = (ROOT / name).read_text(encoding="utf-8")
    require("FUN_180" not in text and "0x" not in text, "reverse-engineering coordinates leaked")
    require("D:\\" not in text and "C:\\" not in text, "absolute path leaked")

print("WASHED_ADM_BWF_RANGE_PACKAGE=PASS")
print("SELECTED_PROFILE_READER_START=UNBOUND")
