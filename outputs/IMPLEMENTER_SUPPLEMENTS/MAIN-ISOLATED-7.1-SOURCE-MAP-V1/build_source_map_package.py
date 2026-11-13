from __future__ import annotations

import hashlib
import json
from pathlib import Path
import zipfile


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "JOCFORGE-MAIN-ISOLATED-7.1-SOURCE-MAP-WASH-V1"
PAYLOADS = (
    "README.md",
    "SOURCE-MAP.md",
    "REUSE-MATRIX.json",
    "SOURCE-RECEIPTS.json",
    "build_source_map_package.py",
    "verify_package.py",
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> None:
    entries = []
    for name in PAYLOADS:
        path = ROOT / name
        data = path.read_bytes()
        entries.append({"path": name, "bytes": len(data), "sha256": sha256(data)})

    manifest = {
        "schema": "jocforge.wash_source_map_manifest.v1",
        "package_id": PACKAGE_ID,
        "payload_count": len(entries),
        "files": entries,
    }
    write_json(ROOT / "MANIFEST.json", manifest)
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    manifest_hash = sha256(manifest_bytes)
    (ROOT / "MANIFEST.sha256").write_text(f"{manifest_hash}  MANIFEST.json\n", encoding="ascii")
    write_json(
        ROOT / "PACKAGE-SEAL.json",
        {
            "schema": "jocforge.wash_source_map_seal.v1",
            "package_id": PACKAGE_ID,
            "manifest_sha256": manifest_hash,
            "payload_count": len(entries),
        },
    )

    archive = ROOT.parent / f"{ROOT.name}.zip"
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as package_zip:
        for path in sorted(ROOT.iterdir()):
            if path.is_file():
                package_zip.write(path, path.relative_to(ROOT).as_posix())
    print(f"PACKAGE_ID={PACKAGE_ID}")
    print(f"MANIFEST_SHA256={manifest_hash}")
    print(f"ZIP_PATH={archive}")
    print(f"ZIP_SHA256={sha256(archive.read_bytes())}")


if __name__ == "__main__":
    main()
