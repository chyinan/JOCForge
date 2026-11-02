from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "ADM-AUTO-TIME-HOST-TO-CONVERTER-MAP-JOIN-V1"
PAYLOADS = ["REPORT.md", "EVIDENCE.json", "INPUT-ONLY-VECTOR.json", "SOURCE-MANIFESTS.json", "verify_package.py", "seal_package.py"]


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def main() -> None:
    files = [{"name": n, "bytes": (ROOT / n).stat().st_size, "sha256": digest(ROOT / n)} for n in PAYLOADS]
    manifest = {"schema": "jocforge.wash_supplement_manifest.v1", "package_id": PACKAGE_ID, "status": "HOST_NORMALIZED_MAP_WRITE_BOUND; CONVERTER_PRIVCONTEXT_TRANSFER_UNBOUND", "recipient_role": "implementation-evidence-supplement", "files": files}
    mp = ROOT / "MANIFEST.json"
    mp.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    mhash = digest(mp)
    (ROOT / "MANIFEST.sha256").write_text(f"{mhash} MANIFEST.json\n", encoding="ascii")
    seal = {"schema": "jocforge.wash_supplement_seal.v1", "package_id": PACKAGE_ID, "manifest_sha256": mhash, "files": files, "seal_excluded_from_own_hash_list": True}
    (ROOT / "package_seal.json").write_text(json.dumps(seal, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(mhash)


if __name__ == "__main__":
    main()
