from __future__ import annotations

import hashlib
import json
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-LFE-ZERO-INLINE-RAW-WORDS-V1"
SOURCE_MANIFEST_SHA256 = "52f001fc2a3215e103899db8358eeb5836b258254acbbd7fc13202e46c3e1eb1"
SOURCE_SEAL_SHA256 = "1a708cbc3195a6a60369e6680ffc25b83cc8f70676537128b7a4a759cc242493"
SOURCE_WORDS_SHA256 = "2468568363d487361c8359e50362ebc5502414e6e3924f2486cb31a71c523f54"
TABLE_EXPECTED = {
    "zero_branch_table_a": ("0x140628c20", 128, 512, "bbd2bd460b0a750ccb45f959561a9a7e2ed2f8c6fd9391fb21abc9216b08fa41"),
    "zero_branch_table_b": ("0x140628e20", 128, 512, "e4cbb855ed455887b387cdb188bf8bab41976c73648c1b022dd18156214bde50"),
}
PAYLOADS = ("EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {PACKAGE_ID}: {message}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build() -> None:
    rows = []
    for name in ("CHECK.py", *PAYLOADS):
        path = ROOT / name
        rows.append({"name": name, "bytes": path.stat().st_size, "sha256": sha256(path)})
    manifest = {
        "schema": "jocforge.wash_supplement_manifest.v1",
        "package_id": PACKAGE_ID,
        "recipient_role": "implementation-evidence-supplement",
        "status": "TWO_ZERO_INLINE_TABLES_256_RAW_WORDS_BOUND; F08_CANDIDATE_COMPARISON_NOT_RUN",
        "implementation_disposition": "Use the two named 128-word vectors as raw binary32 coefficient tables; no candidate match/mismatch claim.",
        "source_manifest_sha256": SOURCE_MANIFEST_SHA256,
        "source_package_seal_sha256": SOURCE_SEAL_SHA256,
        "source_raw_words_sha256": SOURCE_WORDS_SHA256,
        "contains_private_key_or_license": False,
        "contains_pcm_or_media": False,
        "contains_native_source_or_decompilation": False,
        "payloads": rows,
    }
    manifest_path = ROOT / "MANIFEST.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    seal = {
        "schema": "jocforge.wash_supplement_seal.v1",
        "package_id": PACKAGE_ID,
        "status": "SEALED_MANUAL_HASH_AUDIT",
        "manifest_sha256": sha256(manifest_path),
        "payload_file_count": len(rows),
        "payload_hash_audit": f"PASS_{len(rows)}_OF_{len(rows)}",
        "checker_status": "PENDING_NORMAL_AND_PYTHON_O",
    }
    (ROOT / "package_seal.json").write_text(json.dumps(seal, indent=2) + "\n", encoding="utf-8", newline="\n")


def check() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
    require(manifest.get("package_id") == PACKAGE_ID, "package ID mismatch")
    require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST_SHA256, "source manifest mismatch")
    require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL_SHA256, "source seal mismatch")
    require(manifest.get("source_raw_words_sha256") == SOURCE_WORDS_SHA256, "source vector hash mismatch")
    require(seal.get("manifest_sha256") == sha256(manifest_path), "manifest hash mismatch")
    require(seal.get("payload_file_count") == len(manifest.get("payloads", [])), "payload count mismatch")
    expected_names = {"CHECK.py", *PAYLOADS}
    actual_names = {row.get("name") for row in manifest.get("payloads", [])}
    require(actual_names == expected_names, "payload inventory mismatch")
    for row in manifest["payloads"]:
        path = ROOT / row["name"]
        require(path.is_file(), f"missing payload {row['name']}")
        require(path.stat().st_size == row["bytes"], f"byte count mismatch: {row['name']}")
        require(sha256(path) == row["sha256"], f"hash mismatch: {row['name']}")
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    tables = {t["name"]: t for t in vectors["tables"]}
    require(set(tables) == set(TABLE_EXPECTED), "table names mismatch")
    for name, (base, count, byte_count, digest) in TABLE_EXPECTED.items():
        table = tables[name]
        words = table["words"]
        require(table["base_address"] == base, f"base address mismatch: {name}")
        require(len(words) == count == table["word_count"], f"word count mismatch: {name}")
        require(table["byte_count"] == byte_count, f"byte count mismatch: {name}")
        require(table["raw_byte_sha256"] == digest, f"table digest mismatch: {name}")
        require(all(re.fullmatch(r"0x[0-9a-f]{8}", w["raw_word_bits_hex"]) for w in words), f"bad raw word: {name}")
        raw = b"".join(struct.pack("<I", int(w["raw_word_bits_hex"], 16)) for w in words)
        require(len(raw) == byte_count and hashlib.sha256(raw).hexdigest() == digest, f"raw-word digest mismatch: {name}")
        require(all(int(w["address"], 16) == int(base, 16) + 4 * i and w["index"] == i for i, w in enumerate(words)), f"ordering/address mismatch: {name}")
    require(vectors["f08_candidate_comparison"]["status"] == "NOT_RUN_MISSING_128_WORD_CANDIDATE_VALUES", "candidate comparison boundary mismatch")
    require(manifest.get("contains_private_key_or_license") is False, "secret-material flag must be false")
    require(manifest.get("contains_pcm_or_media") is False, "media flag must be false")
    require(manifest.get("contains_native_source_or_decompilation") is False, "native-code flag must be false")
    print(f"PASS package={PACKAGE_ID} tables=2 words=256 candidate_comparison=NOT_RUN")


if __name__ == "__main__":
    if "--build" in sys.argv:
        build()
    else:
        check()
