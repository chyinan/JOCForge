from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    expected = {entry["path"]: entry["sha256"] for entry in manifest["files"]}
    actual = {p.name: sha256(p) for p in ROOT.iterdir() if p.is_file() and p.name not in {"manifest.json", "package_seal.json"}}
    assert actual == expected
    assert seal["manifest_sha256"] == sha256(ROOT / "manifest.json")
    fields = json.loads((ROOT / "FIELD-VALUES.json").read_text(encoding="utf-8"))
    receipts = json.loads((ROOT / "SOURCE-RECEIPTS.json").read_text(encoding="utf-8"))
    assert fields["source_sha256"] == receipts["sample_sha256"]
    assert len(fields["access_units"]) == 2
    assert all(x["order"] == ["I0", "D0"] and x["d0_bytes"] == 2560 for x in fields["access_units"])
    assert fields["d0_bsi_fields"][-1]["name"] == "chanmap"
    assert fields["d0_bsi_fields"][-1]["value"] == "0x1A00"
    assert all(entry["status"].startswith("STABLE") for entry in fields["d0_bsi_fields"])
    assert fields["probe"]["source_bytes_read"] == 22
    assert fields["probe"]["raw_packet_or_frame_bytes_saved"] is False
    assert fields["probe"]["audio_blocks_or_skipfld_parsed"] is False
    assert fields["probe"]["decoder_or_encoder_run"] is False
    assert receipts["sample_sha256_status"].startswith("INHERITED")
    print("PASS official demo D0 first-two-AU BSI vectors; sample-only scope preserved")


if __name__ == "__main__":
    main()
