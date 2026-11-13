from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import zipfile


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "JOCFORGE-MAIN-ISOLATED-7.1-SOURCE-MAP-WASH-V1"
EXPECTED_PAYLOADS = {
    "README.md",
    "SOURCE-MAP.md",
    "REUSE-MATRIX.json",
    "SOURCE-RECEIPTS.json",
    "build_source_map_package.py",
    "verify_package.py",
}
MAIN_ROOT = Path("D:/Programs/JOCForge")
ISOLATED_ROOT = Path("D:/Programs/JOCForge-isolated")


def require(value: bool, reason: str) -> None:
    if not value:
        raise SystemExit(f"PACKAGE_CHECK=FAIL {reason}")


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def git_output(repo: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(repo), *args], text=True).strip()


def check_live_sources(receipts: dict) -> None:
    main = receipts["main_reference"]
    isolated = receipts["isolated_snapshot"]
    require(git_output(MAIN_ROOT, "rev-parse", "main") == main["commit"], "main ref moved")
    require(git_output(ISOLATED_ROOT, "rev-parse", "HEAD") == isolated["head"], "isolated HEAD moved")
    for entry in main["files"]:
        blob = git_output(MAIN_ROOT, "rev-parse", f"main:{entry['path']}")
        require(blob == entry["git_blob"], f"main source changed: {entry['path']}")
    for entry in isolated["files"]:
        data = (ISOLATED_ROOT / entry["path"]).read_bytes()
        require(digest(data) == entry["sha256"], f"isolated source changed: {entry['path']}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--package-only", action="store_true")
    args = parser.parse_args()

    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    manifest_hash = digest(manifest_bytes)
    require(manifest.get("schema") == "jocforge.wash_source_map_manifest.v1", "manifest schema")
    require(manifest.get("package_id") == PACKAGE_ID, "manifest package id")
    seen: set[str] = set()
    for entry in manifest.get("files", []):
        name = entry["path"]
        relative = PurePosixPath(name)
        require(name not in seen, "duplicate payload path")
        require(not relative.is_absolute() and all(part not in {"", ".", ".."} for part in relative.parts), "unsafe payload path")
        payload = ROOT.joinpath(*relative.parts)
        data = payload.read_bytes()
        require(len(data) == entry["bytes"] and digest(data) == entry["sha256"], f"payload hash: {name}")
        seen.add(name)
    actual = {
        path.relative_to(ROOT).as_posix()
        for path in ROOT.rglob("*")
        if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "PACKAGE-SEAL.json"}
    }
    require(seen == EXPECTED_PAYLOADS == actual, "payload inventory")

    seal = load_json(ROOT / "PACKAGE-SEAL.json")
    require(seal.get("schema") == "jocforge.wash_source_map_seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal package id")
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest hash")
    require(seal.get("payload_count") == len(seen), "seal payload count")
    require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split() == [manifest_hash, "MANIFEST.json"], "manifest sidecar")

    receipts = load_json(ROOT / "SOURCE-RECEIPTS.json")
    require(receipts["main_reference"]["commit"] == "05a4108e0c6288130dec1203b301979a91475fca", "main snapshot identity")
    require(receipts["isolated_snapshot"]["head"] == "1acf9c2610edb7da54383481784bf0278e28a0cf", "isolated snapshot identity")
    for entry in receipts["isolated_snapshot"]["files"]:
        require(len(entry["sha256"]) == 64, f"source SHA-256 format: {entry['path']}")

    matrix = load_json(ROOT / "REUSE-MATRIX.json")
    target = matrix["target"]
    require(matrix.get("production_allowed") is False, "production gate")
    require(target["i0_rate_kbps"] == 640 and target["d0_rate_kbps"] == 1024, "per-substream target")
    require(target["aggregate_rate_kbps"] == 1664 and target["d0_chanmap"] == "0x1A00", "aggregate target")
    require(matrix["current_connection_status"]["flat7x_pair_append_called_by_production_cli"] is False, "current route status")

    if not args.package_only:
        check_live_sources(receipts)

    archive = ROOT.parent / f"{ROOT.name}.zip"
    if archive.exists():
        with zipfile.ZipFile(archive) as package_zip:
            require(package_zip.testzip() is None, "ZIP CRC")
            names = set(package_zip.namelist())
            expected_zip = actual | {"MANIFEST.json", "MANIFEST.sha256", "PACKAGE-SEAL.json"}
            require(names == expected_zip, "ZIP inventory")
            for name in expected_zip:
                require(package_zip.read(name) == (ROOT / name).read_bytes(), f"ZIP payload: {name}")

    print("PACKAGE_CHECK=PASS")
    print("SOURCE_SNAPSHOT=PASS" if not args.package_only else "SOURCE_SNAPSHOT=SKIPPED")
    print("PAIR_PRODUCTION_ALLOWED=NO")


if __name__ == "__main__":
    main()
