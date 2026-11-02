from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-36-DBMD-ROW-COUNT-WASH-V1"
PAYLOADS = [
    "REPORT.md",
    "EVIDENCE.json",
    "INPUT-ONLY-VECTOR.json",
    "SOURCE-MANIFESTS.json",
    "verify_package.py",
    "seal_package.py",
]


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            hasher.update(block)
    return hasher.hexdigest()


def main() -> None:
    files = [
        {"name": name, "bytes": (ROOT / name).stat().st_size, "sha256": digest(ROOT / name)}
        for name in PAYLOADS
    ]
    manifest = {
        "schema": "jocforge.wash_supplement_manifest.v1",
        "package_id": PACKAGE_ID,
        "status": "ONE_SAMPLE_DBMD36_CHNA36_FMT36; FIELD_JOIN_AND_RUNTIME_UNBOUND",
        "recipient_role": "implementation-evidence-supplement",
        "files": files,
    }
    manifest_path = ROOT / "MANIFEST.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    manifest_hash = digest(manifest_path)
    (ROOT / "MANIFEST.sha256").write_text(f"{manifest_hash} MANIFEST.json\n", encoding="ascii")
    seal = {
        "schema": "jocforge.wash_supplement_seal.v1",
        "package_id": PACKAGE_ID,
        "manifest_sha256": manifest_hash,
        "files": files,
        "seal_excluded_from_own_hash_list": True,
    }
    (ROOT / "package_seal.json").write_text(json.dumps(seal, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(manifest_hash)


if __name__ == "__main__":
    main()
