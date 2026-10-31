"""Validate the sanitized, hash-bound scalar observation package."""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"expected JSON object: {path.name}")
    return value


def main() -> None:
    root = Path(__file__).resolve().parent
    manifest = read_json(root / "MANIFEST.json")
    evidence = read_json(root / "EVIDENCE.json")

    require(manifest.get("schema") == "wash-evidence-manifest.v1", "manifest schema mismatch")
    require(manifest.get("status") == "FOR_WASH_REVIEW", "status must remain FOR_WASH_REVIEW")
    require(manifest.get("package_id") == "native-measurement-dialnorm-scoped-observation-wash-v1", "package id mismatch")
    require(manifest.get("recipient_role") == "wash-agent", "recipient must remain Wash")
    require(manifest.get("implementation_agent_contacted") is False, "implementation contact must remain false")
    require(manifest.get("contains_raw_addresses") is False, "raw addresses must not be present")
    require(manifest.get("contains_binary_or_decompilation_excerpt") is False, "binary/decompilation must not be present")
    require(manifest.get("contains_pcm_or_bitstream") is False, "audio/bitstream payload must not be present")
    require(manifest.get("contains_private_auth_or_key") is False, "auth/key material must not be present")
    require(manifest.get("contains_local_absolute_paths") is False, "local absolute paths must not be present")

    listed: set[str] = set()
    for entry in manifest.get("files", []):
        name = entry["name"]
        listed.add(name)
        path = root / name
        require(path.is_file(), f"missing payload: {name}")
        require(sha256(path) == entry["sha256"], f"SHA-256 mismatch: {name}")
        require(path.stat().st_size == entry["bytes"], f"byte count mismatch: {name}")
        require(path.suffix.lower() not in {".wav", ".bw64", ".pcm", ".ec3", ".dll", ".exe"}, f"forbidden payload type: {name}")

    expected = {"REPORT.md", "EVIDENCE.json", "validate_scoped_observation_package.py"}
    require(listed == expected, "package file set changed")

    text_files = [root / "MANIFEST.json", *(root / name for name in listed)]
    for path in text_files:
        content = path.read_text(encoding="utf-8")
        require(not re.search(r"(?i)\b[A-Z]:[\\/]", content), f"absolute drive path found: {path.name}")
        require(not re.search(r"(?i)0x[0-9a-f]{6,}", content), f"raw address found: {path.name}")
        require("DECOMPILED" + "_BEGIN" not in content, f"decompilation marker found: {path.name}")

    require(evidence.get("generic_runtime_writer", {}).get("scalar_u16") == 1, "generic scalar changed")
    require(evidence.get("generic_runtime_writer", {}).get("width_bits") == 5, "generic writer width changed")
    require(evidence.get("generic_runtime_writer", {}).get("return_site_matches_static_candidate") is False, "callsite mismatch must remain explicit")
    require(evidence.get("generic_runtime_writer", {}).get("source_field_value_in_same_call") == "NOT_OBSERVED", "candidate source must remain unobserved")
    require(evidence.get("measurement_chain", {}).get("measurement_to_f06nd_drc_input") == "UNBOUND", "measurement/DRC join must remain unbound")
    require(evidence.get("measurement_chain", {}).get("f06nd_drc_to_f12_bsi_dialnorm") == "UNBOUND", "DRC/BSI join must remain unbound")
    require(evidence.get("measurement_chain", {}).get("fresh_finalize_observer") == "NOT_RUN", "fresh finalize observer must remain NOT_RUN")
    require(evidence.get("full_e2e_and_player_playback") == "NOT_RUN", "playback must remain NOT_RUN")
    require(evidence.get("run_metadata", {}).get("output_payload_read_or_retained_by_observer") is False, "payload must not be read or retained")
    require(evidence.get("private_auth_or_key_contents_read") is False, "auth/key contents must remain unread")
    require(evidence.get("implementation_agent_contacted") is False, "implementation contact must remain false")

    print("package_files=HASH_AND_SIZE_MATCH")
    print("sanitization=NO_PATHS_ADDRESSES_DECOMP_BINARY_AUDIO_OR_AUTH")
    print("generic_width5_scalar=OBSERVED_1; STATIC_CANDIDATE_MATCH=NO")
    print("measurement_to_dialnorm_join=UNBOUND; PLAYBACK=NOT_RUN")


if __name__ == "__main__":
    main()
