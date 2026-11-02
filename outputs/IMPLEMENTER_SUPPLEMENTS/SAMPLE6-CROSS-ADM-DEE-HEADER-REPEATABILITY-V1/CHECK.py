import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-CROSS-ADM-DEE-HEADER-REPEATABILITY-V1"


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
    require((ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0] == manifest_sha, "manifest hash")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    files = manifest.get("files")
    require(isinstance(files, list) and files, "manifest files")
    for item in files:
        name = item.get("name")
        require(isinstance(name, str) and Path(name).name == name, "safe member")
        member = ROOT / name
        require(member.is_file() and digest(member) == item.get("sha256"), f"member hash {name}")
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == manifest_sha, "seal manifest")
    require(seal.get("files") == files, "seal members")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(len(evidence["adm_inputs"]) == 3, "input count")
    require(all(item["hash_recomputed_by_wash"] for item in evidence["adm_inputs"]), "ADM hashes")
    require(all(item["exit_code"] == 0 for item in evidence["additional_dee_run_receipts"]), "DEE exits")
    require(evidence["repeated_adm_metadata"]["lfe_row"] == 3, "LFE row")
    require(evidence["limits"]["per_channel_pcm_metrics"] == "NOT_RUN", "no PCM claim")
    require(evidence["limits"]["key_or_license_contents"] is False, "no key")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_sha}")
    print("PASS source identities, run receipts, and metadata-only scope")


if __name__ == "__main__":
    main()
