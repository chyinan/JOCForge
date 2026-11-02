import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-MATCHED-FULL-FILE-DEE-PCM-COMPARISON-V1"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest_hash = sha256(manifest_path)
    declared_hash = (ROOT / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    require(manifest_hash == declared_hash, "manifest hash")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    files = manifest.get("files")
    require(isinstance(files, list) and files, "manifest files")
    for entry in files:
        name = entry.get("name")
        require(isinstance(name, str) and Path(name).name == name, "safe member name")
        member = ROOT / name
        require(member.is_file(), f"member exists: {name}")
        require(sha256(member) == entry.get("sha256"), f"member hash: {name}")

    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    require(seal.get("package_id") == PACKAGE_ID, "seal package id")
    require(seal.get("manifest_sha256") == manifest_hash, "seal manifest hash")
    require(seal.get("files") == files, "seal member hashes")

    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    controls = json.loads((ROOT / "DRC-SCALE-CONTROL.json").read_text(encoding="utf-8"))
    require(evidence["input"]["original_sha256"] == "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1", "authorized ADM identity")
    require(evidence["matched_job_controls"]["dialogue_intelligence"] is False, "matched DI control")
    require(evidence["matched_job_controls"]["speech_threshold"] == 100, "matched speech threshold")
    require(evidence["dee_output"]["sha256"] == "80c3b9b9a3ed1eaa74a53dbd5bbdc714b0f00c1501e44e4c43eb1aa4ca9a2662", "DEE output identity")
    require(evidence["rust_output"]["sha256"] == "d786ac4dcef680e2fb3a4fee7e52bf9b82e9d3585b5b09e2dde2a521e37cf11b", "Rust output identity")
    require(evidence["comparison_window"]["au_interval_half_open"] == [2750, 2875], "AU interval")
    require(evidence["comparison_window"]["frames_per_channel"] == 192000, "sample count")
    require(len(controls["results"]) == 2, "DRC-scale controls")
    scale_one, scale_zero = controls["results"]
    require(scale_one["drc_scale"] == 1 and scale_zero["drc_scale"] == 0, "DRC-scale values")
    require(scale_one["rust_window_pcm_sha256"] == "6d9a5de71c530e65d87aa7ef203b3900ff50beec336018c7b269ed061747f2f7", "Rust PCM window hash")
    require(scale_zero["per_channel"][3]["rmse"] > 0.0054, "channel 3 residual")
    require(all(channel["rmse"] < 0.0005 for channel in scale_zero["per_channel"] if channel["index"] != 3), "other five channels")
    require(evidence["privacy_and_method"]["raw_pcm_or_encoded_payload_included"] is False, "no raw media")
    require(evidence["privacy_and_method"]["license_or_key_contents_retained"] is False, "no secret material")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS source/input/output/window identities")
    print("PASS in-memory scalar metrics and DRC-scale residual localization")
    print("PASS no raw PCM/payload/private-key material")


if __name__ == "__main__":
    main()
