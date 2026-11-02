import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-DEE-MATCHED-LFE-POINTER-EDGE-V1"


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
    vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
    require(evidence["authorized_adm_metadata"]["source_row"] == 3, "ADM row")
    require(evidence["dee_static_typed_path"]["request_input"].startswith("six-channel"), "typed request")
    require(evidence["dee_static_typed_path"]["first_sample_operation"].startswith("internal category 3"), "LFE unity path")
    require(evidence["dee_static_typed_path"]["same_run_join"].startswith("not captured"), "owner join limit")
    require(vector["execution_status"] == "NOT_RUN", "vector execution boundary")
    require(evidence["limits"]["corrective_gain_or_remap_supported"] is False, "no guessed fix")
    require(evidence["limits"]["pcm_or_payload_included"] is False, "no raw media")
    require(evidence["limits"]["private_key_or_license_material"] is False, "no key")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_sha}")
    print("PASS typed slot path, sample operation, and exact UNBOUND limits")


if __name__ == "__main__":
    main()
