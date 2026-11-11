#!/usr/bin/env python3
"""Build a deterministic seal for the limited Flat7x D0 BSI mixing segment."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "eac3-dependent-d0-mixing-metadata-v1"
STATUS = "IMPLEMENTER_READY_LIMITED_D0_MIXING_INFO_SEGMENT"
FILES = (
    "CONTRACT.md",
    "CONTRACT.json",
    "INPUT-ONLY-VECTORS.json",
    "PROVENANCE.json",
    "build_manifest.py",
    "verify_package.py",
)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = {
    "files": [
        {"path": name, "sha256": digest(ROOT / name), "size_bytes": (ROOT / name).stat().st_size}
        for name in FILES
    ],
    "package_id": PACKAGE_ID,
    "production_allowed": False,
    "schema": "jocforge.wash_implementer_supplement.v1",
    "status": STATUS,
}
manifest_bytes = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")
(ROOT / "MANIFEST.json").write_bytes(manifest_bytes)
seal = {
    "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
    "package_id": PACKAGE_ID,
    "payload_count": len(FILES),
    "production_allowed": False,
    "schema": "jocforge.wash_implementer_supplement_seal.v1",
}
(ROOT / "PACKAGE-SEAL.json").write_text(json.dumps(seal, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"SEALED package={PACKAGE_ID} payloads={len(FILES)} production_allowed=false")
