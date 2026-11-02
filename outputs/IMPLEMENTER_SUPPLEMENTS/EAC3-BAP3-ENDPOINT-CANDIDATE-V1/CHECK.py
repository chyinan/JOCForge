import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "EAC3-BAP3-ENDPOINT-CANDIDATE-V1"


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
    require(seal.get("files") == files, "seal member hashes")
    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
    require(evidence["native_behavior"]["positive_endpoint"]["clamped_code"] == 6, "native clamp")
    require(vector["expected_native"]["packed_bits"] == "110", "native vector result")
    require(vector["expected_current_model"]["disposition"].startswith("reserved sentinel"), "current model boundary")
    require(evidence["scope"]["occurrence_in_matched_human_cat_window"] == "UNOBSERVED", "sample occurrence boundary")
    require(vector["execution"] == "NOT_RUN", "execution status")
    require(evidence["privacy"]["private_key_or_license_material"] is False, "no key")
    require(evidence["privacy"]["raw_pcm"] is False, "no PCM")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_sha}")
    print("PASS native/Wash endpoint split, input-only vector, and NOT_RUN boundaries")


if __name__ == "__main__":
    main()
