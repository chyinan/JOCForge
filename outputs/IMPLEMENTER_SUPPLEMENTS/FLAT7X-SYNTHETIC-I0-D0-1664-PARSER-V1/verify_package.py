from __future__ import annotations

import hashlib
import json
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "FLAT7X-SYNTHETIC-I0-D0-1664-PARSER-V1"
EXPECTED = {"FINDINGS.md", "STRUCTURAL-CONTRACT.json", "SOURCE-RECEIPTS.json", "verify_package.py"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"PACKAGE_CHECK=FAIL {message}")


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes)
    require(manifest.get("schema") == "jocforge.wash_implementer_supplement.v1", "manifest schema")
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    records = manifest.get("files")
    require(isinstance(records, list), "manifest files")
    found = set()
    for record in records:
        name = record.get("path")
        require(isinstance(name, str) and name not in found, "duplicate or invalid path")
        relative = PurePosixPath(name)
        require(not relative.is_absolute() and all(part not in {"", ".", ".."} for part in relative.parts), "unsafe path")
        path = ROOT.joinpath(*relative.parts)
        require(path.is_file() and path.stat().st_size == record.get("bytes"), f"size: {name}")
        require(digest(path) == record.get("sha256"), f"hash: {name}")
        found.add(name)
    actual = {path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*") if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "PACKAGE-SEAL.json"}}
    require(found == EXPECTED == actual, "payload inventory")
    seal = load("PACKAGE-SEAL.json")
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == hashlib.sha256(manifest_bytes).hexdigest(), "seal digest")
    require(seal.get("payload_count") == len(records), "seal count")
    require(seal.get("file_paths") == sorted(EXPECTED), "seal paths")
    sidecar = (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(sidecar == [hashlib.sha256(manifest_bytes).hexdigest(), "MANIFEST.json"], "manifest sidecar")

    contract = load("STRUCTURAL-CONTRACT.json")
    require(contract.get("classification") == "SYNTHETIC_NOT_REAL_OUTPUT", "classification")
    require(contract.get("production_allowed") is False, "production gate")
    au = contract.get("target_access_unit", {})
    require(au.get("composition_order") == ["I0", "D0"] and au.get("total_bytes") == 6656, "AU composition")
    require(au.get("aggregate_bitrate_bps") == 1664000 and au.get("samples") == 1536, "AU target")
    i0 = contract.get("i0", {})
    require(i0.get("bsid") == 6 and i0.get("frame_size_bytes") == 2560 and i0.get("crc_validation") == "NOT_RUN", "I0 boundary")
    d0 = contract.get("d0", {})
    require(d0.get("bsid") == 16 and d0.get("frame_size_bytes") == 4096 and d0.get("chanmap") == "0x1A00", "D0 boundary")
    field = d0.get("block2_skipfld", {})
    require(field.get("field_count") == 1 and field.get("skipl_bytes") == field.get("C_total_bytes") == 511, "single field C511")
    require(field.get("bytes_consumed") == 511 and field.get("exact_carrier_result") == "Parsed", "exact carrier")
    require(field.get("joc_downmix_index") == 1 and field.get("joc_channel_count") == 7 and field.get("joc_objects_present") == 15, "JOC structure")
    boundaries = contract.get("open_boundaries", {})
    require(boundaries.get("complexity16") == "NOT_INFERRED" and boundaries.get("audio_decoder") == "NOT_RUN", "unresolved boundaries")
    receipts = load("SOURCE-RECEIPTS.json")
    require(receipts.get("source_package", {}).get("manifest_sha256") == "1ab53a27fd3f632c2579c6dea2830638ce0f9b99cddc957564f38086671eecde", "source manifest binding")
    require(receipts.get("source_package", {}).get("sha256sums_sha256") == "2a1a4a815717b3c0f97844d3a0c512c6301e46be21ce9a62772d28e2aa84a404", "source sums binding")
    require(receipts.get("access", {}).get("raw_frame_or_payload_bytes_included") is False, "payload exclusion")
    print("PACKAGE_CHECK=PASS")
    print("CLASSIFICATION=SYNTHETIC_NOT_REAL_OUTPUT")
    print("STRUCTURAL_PARSER_SCOPE=I0_D0_GROUPING_AND_D0_BLOCK2_C511_JOC15")
    print("PRODUCTION_ALLOWED=NO")


if __name__ == "__main__":
    main()
