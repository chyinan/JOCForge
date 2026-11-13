from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-D0-BLOCK-INDEX-CONVENTION-WASH-V1"
EXPECTED_FILES = {"EVIDENCE.json", "FINDINGS.md", "verify_package.py"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


manifest_path = ROOT / "MANIFEST.json"
seal_path = ROOT / "PACKAGE-SEAL.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
seal = json.loads(seal_path.read_text(encoding="utf-8"))

require(manifest.get("package") == PACKAGE_ID, "unexpected package identity")
require(seal.get("package") == PACKAGE_ID, "unexpected seal identity")
require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest digest mismatch")

entries = manifest.get("files")
require(isinstance(entries, list), "manifest files must be a list")
listed = {entry.get("path") for entry in entries if isinstance(entry, dict)}
require(listed == EXPECTED_FILES, "manifest file inventory mismatch")
for entry in entries:
    path = ROOT / entry["path"]
    require(path.is_file(), f"missing file: {entry['path']}")
    require(path.stat().st_size == entry.get("bytes"), f"size mismatch: {entry['path']}")
    require(sha256(path) == entry.get("sha256"), f"hash mismatch: {entry['path']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
resolution = evidence["resolution"]
require(resolution["contract_d0_emdf_block_index"] == 2, "target contract index mismatch")
require(resolution["contract_index_base"] == "ZERO_BASED", "target contract base mismatch")
require(resolution["parser_block_index"] == 2 and resolution["parser_index_base"] == "ZERO_BASED", "parser index mismatch")
require(resolution["audio_block_prefix_position_one_based"] == 3, "human sequence position mismatch")
require(resolution["same_slot"] is True, "contract/parser slot join mismatch")
require(evidence["wire_field_status"]["parser_rerun"] is False, "parser rerun must remain false")
require(evidence["wire_field_status"]["raw_bytes_read_or_exported"] is False, "raw bytes must remain excluded")
require(evidence["official_sample_receipt"]["target_behavior_inferred"] is False, "sample-to-target inference must remain false")

print("PASS: Flat7x D0 block-index convention reconciliation")
