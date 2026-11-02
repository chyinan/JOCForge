from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ATMOS-PREFERRED-DOWNMIX-CONSUMER-BOUNDARY-V1"
SOURCE_MANIFEST = "4de3c1af1b49eb8a70c9fa8a574bbfb5dbbb9882d3cc020455b6624208b65577"
STATUS = "PARSER_FIELD_READ; GENERIC_RESULT_BRANCH; FINAL_F13C_CONSUMER_UNBOUND"


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> None:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID and manifest["status"] == STATUS, "manifest id/status")
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST, "source manifest")
    entries = {entry["file"]: entry for entry in manifest["files"]}
    actual = {
        path.name for path in ROOT.iterdir()
        if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
    }
    require(actual == set(entries), "payload inventory")
    for name, entry in entries.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID, "seal id")
    require(set(sealed) == set(entries) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")
    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    fields = evidence["native_parser"]["parsed_fields"]
    require(fields == ["lorocmixlev", "lorosurmixlev", "dmixmod", "ltrtcmixlev", "ltrtsurmixlev"], "parser fields")
    require(evidence["native_parser"]["preferred_downmix_mode_to_dmixmod_alias"] == "UNBOUND", "alias boundary")
    require(evidence["branch_boundary"]["downstream_audio_consumer"] == "UNBOUND", "consumer boundary")
    require(evidence["effect_on_pcm_goal"]["should_block_window_table_trial"] is False, "audio trial disposition")
    require(evidence["scope"]["encoder_run"] == "NOT_RUN" and evidence["scope"]["pcm_or_media_read"] is False, "scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS parser/result-tree boundary and downstream audio consumer UNBOUND")


if __name__ == "__main__":
    main()
