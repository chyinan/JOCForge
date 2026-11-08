from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "JOCFORGE-ATMOS71-FLAT7X-1664-PROFILE-CONTRACT-V1"
STATUS = "WASH_APPROVED_TARGET_CONTRACT; PRODUCTION_AUDIO_NOT_READY"
EXCLUDED = {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {message}")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(name: str) -> dict:
    value = json.loads((ROOT / name).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"JSON object {name}")
    return value


def main() -> None:
    manifest_bytes = (ROOT / "MANIFEST.json").read_bytes()
    digest = sha256(manifest_bytes)
    manifest = json.loads(manifest_bytes)
    require(manifest.get("schema") == "jocforge-profile-contract-supplement.v1", "manifest schema")
    require(manifest.get("package_id") == PACKAGE_ID, "package id")
    require(manifest.get("status") == STATUS, "package status")
    require(manifest.get("implementation_ready") == "NO_AUDIO_PRODUCTION", "implementation gate")
    files = manifest.get("files")
    require(isinstance(files, list) and files, "payload files")
    names = [item.get("path") for item in files]
    require(all(isinstance(name, str) for name in names), "payload paths")
    require(len(names) == len(set(names)), "unique payload paths")
    actual = sorted(
        item.relative_to(ROOT).as_posix()
        for item in ROOT.rglob("*")
        if item.is_file() and item.name not in EXCLUDED
    )
    require(actual == sorted(names), "exact payload inventory")
    for item in files:
        name = item["path"]
        require(Path(name).suffix.lower() in {".json", ".md", ".py"}, f"artifact type {name}")
        if name.endswith(".py"):
            require(name == "CHECK.py", "only package checker script")
        data = (ROOT / name).read_bytes()
        require(len(data) == item.get("bytes"), f"length {name}")
        require(sha256(data) == item.get("sha256"), f"payload hash {name}")
    require(
        (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").strip()
        == f"{digest} MANIFEST.json",
        "detached manifest digest",
    )
    seal = read_json("package_seal.json")
    require(seal.get("schema") == "jocforge-profile-contract-seal.v1", "seal schema")
    require(seal.get("package_id") == PACKAGE_ID, "seal id")
    require(seal.get("manifest_sha256") == digest, "seal manifest hash")
    require(seal.get("files") == files, "seal inventory")

    row = read_json("PROFILE-ROW.json")
    require(row["status"] == "PROFILE_CONTRACT_APPROVED; PRODUCTION_AUDIO_NOT_READY", "profile row status")
    require(row["production_gates"]["production_allowed"] is False, "production gate")
    require(row["input_scope"]["arbitrary_n_expansion"] == "DEFERRED", "N scope")
    require(row["i0"]["rate_kbps"] == 640 and row["i0"]["frame_bytes"] == 2560, "I0 rate/size")
    require(row["d0"]["rate_kbps"] == 1024 and row["d0"]["frame_bytes"] == 4096, "D0 rate/size")
    require(row["d0"]["chanmap_hex"] == "0x1A00", "D0 map")
    require(row["d0"]["coded_channel_locations"] == ["Ls", "Rs", "Lrs", "Rrs"], "D0 channel order")
    require(row["joc"]["reconstruction_profile"] == "flat7x", "JOC profile")
    require(row["joc"]["profile_index"] == 1 and row["joc"]["reconstruction_carrier_count"] == 7, "JOC index/geometry")
    require(row["joc"]["coded_object_count"] == 15 and row["joc"]["reconstruction_rows"] == 15, "JOC object rows")
    require(row["joc"]["complexity_index"] == 16, "JOC complexity")
    require(row["access_unit"]["ordered_substreams"] == ["I0", "D0"], "AU order")
    require(row["access_unit"]["total_frame_bytes"] == 6656, "AU bytes")
    require(row["access_unit"]["aggregate_rate_kbps"] == 1664, "aggregate rate")
    require(row["emdf"]["maximum_complete_container_bytes"] == 511, "EMDF cap")
    require(row["emdf"]["cross_skipfld_fragmentation"] == "FORBIDDEN", "EMDF fragmentation")
    require(row["emdf"]["all_au_fit_status"] == "NOT_RUN", "EMDF fit status")
    require(row["production_gates"]["gtc_source_role_projection"] == "UNBOUND", "GTC role gate")
    require(row["production_gates"]["native_d0_audio_algorithm"] == "UNBOUND", "D0 algorithm gate")

    domains = read_json("COUNT-DOMAINS-AND-GTC-PROJECTION.json")
    counts = {item["domain"]: item["count"] for item in domains["count_domains"]}
    require(counts["I0 coded channels"] == 6, "I0 domain")
    require(counts["D0 coded channel locations"] == 4, "D0 domain")
    require(counts["JOC reconstruction carriers"] == 7, "JOC domain")
    require(counts["ADM physical BWF rows"] == "N, presently 36/38 scope", "ADM row domain")
    require(domains["gtc_owner_projection_contract"]["current_input_specific_owner_mapping"] == "UNBOUND", "GTC join")

    disposition = read_json("EMDF-AND-VALIDATION-DISPOSITION.json")
    require(disposition["target_carriage"]["continuation_into_next_skipfld"] == "forbidden", "target carrier rule")
    require(disposition["target_carriage"]["block_zero_based"] == 2, "target block")
    require(disposition["target_carriage"]["target_emdf_fit_for_all_generated_aus"] == "NOT_RUN", "target fit status")
    require(disposition["observed_official_reference"]["observed_vendor_compat"] == "PASS", "vendor status")
    require(disposition["observed_official_reference"]["etsi_strict"] == "FAIL_PAYLOAD_CONFIGURATION_FLAGS", "strict status")

    receipts = read_json("SOURCE-RECEIPTS.json")
    require(receipts["draft_package"]["checker_normal"] == "PASS", "draft checker normal")
    require(receipts["draft_package"]["checker_python_optimized"] == "PASS", "draft checker optimized")
    require(receipts["isolated_checkout_review"]["current_source_file_hash_matches"] == 29, "source file revalidation")
    require(receipts["isolated_checkout_review"]["working_tree_dirty"] is True, "dirty worktree preserved")
    require(receipts["isolated_checkout_review"]["wash_modified_checkout"] is False, "no checkout edits")
    restricted = receipts["restricted_inputs"]
    require(restricted["official_mka_path_or_hash_forwarded"] is False, "official media excluded")
    require(restricted["mode21_ec3_path_hash_or_bytes_forwarded"] is False, "mode21 payload excluded")
    require(restricted["historical_main_source_code_forwarded"] is False, "main code excluded")
    require(restricted["private_key_or_license_data"] is False, "private data excluded")
    report = (ROOT / "WASH-REVIEW.md").read_text(encoding="utf-8")
    require("UNBOUND" in report and "NOT_RUN" in report, "open states retained")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={digest}")
    print("PASS target row, distinct count domains, D0 gates, and clean-room exclusions")


if __name__ == "__main__":
    main()
