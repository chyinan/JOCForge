from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "eac3-dependent-chanmap-field-writer-v1"
PAYLOADS = (
    "CONTRACT.md",
    "CONTRACT.json",
    "INPUT-ONLY-VECTORS.json",
    "PROVENANCE.json",
    "build_manifest.py",
    "verify_package.py",
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


entries = [
    {"path": name, "size_bytes": (ROOT / name).stat().st_size, "sha256": sha256(ROOT / name)}
    for name in PAYLOADS
]
manifest = {
    "schema": "jocforge.wash_implementer_supplement.v1",
    "package_id": PACKAGE_ID,
    "status": "IMPLEMENTER_READY_LIMITED_FIELD_SERIALIZER",
    "production_allowed": False,
    "files": entries,
}
manifest_bytes = (json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")
(ROOT / "MANIFEST.json").write_bytes(manifest_bytes)
seal = {
    "schema": "jocforge.wash_implementer_supplement_seal.v1",
    "package_id": PACKAGE_ID,
    "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
    "payload_count": len(entries),
    "production_allowed": False,
}
(ROOT / "PACKAGE-SEAL.json").write_text(
    json.dumps(seal, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
    newline="\n",
)
print(f"MANIFEST_SHA256={hashlib.sha256(manifest_bytes).hexdigest()}")
print(f"PAYLOAD_COUNT={len(entries)}")
