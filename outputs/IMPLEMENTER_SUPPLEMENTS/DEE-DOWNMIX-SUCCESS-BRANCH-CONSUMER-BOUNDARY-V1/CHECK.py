from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-DOWNMIX-SUCCESS-BRANCH-CONSUMER-BOUNDARY-V1"
SOURCE_MANIFEST = "68f8e000a1d74786da60ceedb2e73925355c05aa712ed679eba1432ba964e6a5"
STATUS = "DOWNMIX_PARSER_SLOTS_BOUND; FIRST_TYPED_AUDIO_CONSUMER_UNBOUND"


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
    require(evidence["parser_output_slots"]["dmixmod"] == "+0x1b0", "dmixmod slot")
    require(evidence["result"]["dmixmod_to_audio_matrix"] == "UNBOUND", "consumer boundary")
    require(evidence["result"]["fields_ignored_downstream"] == "NOT_ESTABLISHED", "no false negative claim")
    require(evidence["result"]["pcm_mismatch_explained"] is False, "PCM scope")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS bounded slot scan and explicit audio-consumer UNBOUND disposition")


if __name__ == "__main__":
    main()
