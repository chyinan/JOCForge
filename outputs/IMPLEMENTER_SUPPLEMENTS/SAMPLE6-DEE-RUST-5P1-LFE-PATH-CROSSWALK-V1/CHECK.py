import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-DEE-RUST-5P1-LFE-PATH-CROSSWALK-V1"


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
    require(evidence["dee_static_path"]["physical_adm_rows_0_to_5"][3] == "LFE", "ADM row 3 label")
    require(evidence["dee_static_path"]["six_channel_core_api_order"][3] == "LFE", "core slot 3 label")
    require(evidence["dee_static_path"]["lfe_category"] == 3, "DEE internal LFE category")
    require(evidence["limits"]["runtime_38_to_6_row_selection"] == "NOT_CAPTURED", "runtime selection boundary")
    require(evidence["limits"]["native_encoder_internal_lfe_transform"] == "UNBOUND", "core transform boundary")
    require(evidence["limits"]["corrective_gain_or_remap_supported"] is False, "no guessed correction")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_sha}")
    print("PASS static row/core order, Rust source hashes, and UNBOUND runtime limits")


if __name__ == "__main__":
    main()
