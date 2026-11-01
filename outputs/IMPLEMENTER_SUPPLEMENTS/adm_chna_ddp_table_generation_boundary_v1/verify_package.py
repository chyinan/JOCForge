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
require(manifest.get("schema") == "washed-adm-chna-ddp-table-generation-boundary.v1", "schema mismatch")
require(manifest.get("status") == "CHNA_TO_ZERO_BASED_TRACK_SLOT_STATIC; READER_TO_DDP_ROW_GENERATION_UNBOUND", "status mismatch")

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
require(obs.get("implementation_disposition", {}).get("end_to_end_ddp_row_permutation_approved") is False, "DDP permutation must remain unapproved")
require(obs.get("input", {}).get("full_file_sha256") == "NOT_COMPUTED", "source hash scope mismatch")
require(obs.get("runtime_scope", {}).get("new_dee_job") == "NOT_RUN", "runtime status mismatch")
require("same parent job and generation" in " ".join(obs.get("unbound", [])), "generation boundary missing")
for name in ("WASH-REVIEW.md", "OBSERVATION.json", "VERIFICATION.json"):
    text = (ROOT / name).read_text(encoding="utf-8")
    require("FUN_180" not in text and "0x" not in text, "reverse-engineering coordinates leaked")
    require("D:\\" not in text and "C:\\" not in text, "absolute path leaked")

print("WASHED_ADM_CHNA_DDP_BOUNDARY=PASS")
print("READER_TO_DDP_ROW_GENERATION=UNBOUND")
