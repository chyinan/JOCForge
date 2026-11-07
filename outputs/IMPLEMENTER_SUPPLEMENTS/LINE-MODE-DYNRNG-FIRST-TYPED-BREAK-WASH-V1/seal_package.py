import hashlib
import json
from pathlib import Path


def main() -> None:
    package_dir = Path(__file__).resolve().parent
    artifacts = []
    for path in sorted(package_dir.rglob("*")):
        if not path.is_file() or path.name in {"MANIFEST.json", "MANIFEST.sha256"}:
            continue
        payload = path.read_bytes()
        artifacts.append(
            {
                "path": path.relative_to(package_dir).as_posix(),
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    manifest = {
        "schema": "jocforge.wash_package_manifest.v1",
        "package_id": "LINE-MODE-DYNRNG-FIRST-TYPED-BREAK-WASH-V1",
        "delivery": "WASH_ONLY",
        "artifacts": artifacts,
    }
    encoded = (json.dumps(manifest, indent=2, ensure_ascii=False) + "\n").encode("utf-8")
    (package_dir / "MANIFEST.json").write_bytes(encoded)
    digest = hashlib.sha256(encoded).hexdigest()
    (package_dir / "MANIFEST.sha256").write_text(f"{digest}  MANIFEST.json\n", encoding="ascii")
    print(digest)


if __name__ == "__main__":
    main()
