from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


manifest_raw = (ROOT / "MANIFEST.json").read_bytes()
expected_manifest_hash = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0]
require(hashlib.sha256(manifest_raw).hexdigest() == expected_manifest_hash, "manifest hash mismatch")
manifest = json.loads(manifest_raw)
require(manifest.get("schema") == "washed-conditional-pcm24-s16-branch.v1", "schema mismatch")
require(manifest.get("status") == "CONDITIONAL_STATIC_BRANCH_NOT_RUNTIME_JOINED", "status mismatch")

for entry in manifest.get("payloads", []):
    name = entry.get("file")
    require(isinstance(name, str) and name, "payload name missing")
    path = (ROOT / name).resolve()
    require(path.is_relative_to(ROOT.resolve()), "payload escapes package")
    require(path.is_file(), f"payload missing: {name}")
    payload = path.read_bytes()
    require(len(payload) == entry.get("bytes"), f"payload size mismatch: {name}")
    require(hashlib.sha256(payload).hexdigest() == entry.get("sha256"), f"payload hash mismatch: {name}")

obs = json.loads((ROOT / "OBSERVATION.json").read_text(encoding="utf-8"))
require(obs.get("implementation_disposition", {}).get("production_rule_approved") is False, "candidate must not be implementation-authoritative")
require(obs.get("scope", {}).get("runtime_execution") == "NOT_RUN", "runtime status must remain explicit")
require(obs.get("scope", {}).get("pcm_samples_read_or_persisted") is False, "PCM claim mismatch")
require("first F32 DSP consumer" in " ".join(obs.get("unbound", [])), "F32 boundary must remain open")
for name in ("WASH-REVIEW.md", "OBSERVATION.json", "VERIFICATION.json"):
    content = (ROOT / name).read_text(encoding="utf-8")
    require("FUN_180" not in content and "0x236e40" not in content, "raw reverse-engineering coordinates leaked")
    require("D:\\" not in content and "C:\\" not in content, "absolute path leaked")

print("WASHED_PCM24_BRANCH_PACKAGE=PASS")
print("PRODUCTION_PCM24_TO_F32_RULE=NOT_APPROVED")
