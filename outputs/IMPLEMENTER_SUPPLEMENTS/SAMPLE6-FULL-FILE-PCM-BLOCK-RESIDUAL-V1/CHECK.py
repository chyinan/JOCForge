import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-FULL-FILE-PCM-BLOCK-RESIDUAL-V1"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest_sha = digest(manifest_path)
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0] == manifest_sha,
        "manifest hash",
    )
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    files = manifest.get("files")
    require(isinstance(files, list) and files, "manifest files")
    for item in files:
        name = item.get("name")
        require(isinstance(name, str) and Path(name).name == name, "safe member")
        path = ROOT / name
        require(path.is_file() and digest(path) == item.get("sha256"), f"member hash {name}")
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == manifest_sha, "seal manifest")
    require(seal.get("files") == files, "seal member hashes")

    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    channel3 = evidence["whole_file_channel_metrics"][3]
    block = evidence["four_second_block_summary"]
    require(channel3["channel_index"] == 3, "channel index")
    require(channel3["rmse"] > 0.006, "whole-file channel-3 RMSE")
    require(block["channel_3_rmse_rank_first_blocks"] == 30, "block dominance count")
    require(evidence["privacy"]["raw_pcm_or_payload_included"] is False, "no raw media")
    require(evidence["privacy"]["private_key_or_license_material_included"] is False, "no key")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_sha}")
    print("PASS full-file identities, channel-3 persistence, and privacy bounds")


if __name__ == "__main__":
    main()
