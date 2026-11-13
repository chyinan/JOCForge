from __future__ import annotations

import hashlib
import json
from pathlib import Path

PACKAGE_DIR = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-D0-FOUR-CHANNEL-ENCODER-POLICY-V1"
BASE_COMMIT = "1099a56115e659415ef2dcc1d373046b1bd33fdf"
CONTENT_FILES = [
    "CONTRACT.md",
    "POLICY.json",
    "INPUT-ONLY-VECTORS.json",
    "SOURCE-REFERENCES.json",
    "CHECK.py",
    "build_seal.py",
]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest = {
    "schema": "wash-flat7x-d0-encoder-policy-manifest.v1",
    "package_id": PACKAGE_ID,
    "status": "PROVISIONAL_IMPLEMENTATION_CONTRACT",
    "base_isolated_commit": BASE_COMMIT,
    "files": [{"path": name, "sha256": digest(PACKAGE_DIR / name)} for name in CONTENT_FILES],
    "source_references_sha256": digest(PACKAGE_DIR / "SOURCE-REFERENCES.json"),
    "execution_status": "NOT_RUN",
    "production_allowed": False,
}
manifest_path = PACKAGE_DIR / "MANIFEST.json"
manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
seal_files = {
    path.name: digest(path)
    for path in PACKAGE_DIR.iterdir()
    if path.is_file() and path.name != "PACKAGE-SEAL.json"
}
seal = {
    "schema": "wash-flat7x-d0-encoder-policy-seal.v1",
    "package_id": PACKAGE_ID,
    "manifest_sha256": digest(manifest_path),
    "files": seal_files,
    "seal_excluded_from_its_own_hash_list": True,
}
(PACKAGE_DIR / "PACKAGE-SEAL.json").write_text(
    json.dumps(seal, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
print(f"sealed {PACKAGE_ID} files={len(seal_files)}")
