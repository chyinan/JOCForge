from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True

PACKAGE_ID = "dee_atmos_ddp_silence_first_consumer_wash_v1"
RULE_PACKAGE_ID = "dee_atmos_ddp_silence_duration_sample_rule_wash_v1"
DLL_SHA256 = "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2"
CALLEE_RVA = 0x1844C0
CALLEE_FILE_OFFSET = 0x1838C0
CALLEE_BYTES = 1037
CALLEE_SHA256 = "1f6d40dd98ea38779e6c02c8f4ccdc8e312aff9ed2a52082f26e53acf9fa1d0c"
RULE_MANIFEST_SHA256 = "95f2282edfbf32fbbf6f4c72662242c400429b44fdc32396bf595c8367fb9220"
RULE_SEAL_SHA256 = "4b617e06a3fd7ede6c5707fac4bddb2d57a7bd32ce669dabaa812faa5346aaca"
EXPECTED_STATUS = "FIRST_COMPOSITE_RECORD_CONSUMER_BOUND; COUNTS_COPIED_UNCHANGED; PCM_OR_PTS_EFFECT_UNBOUND; RUNTIME_NOT_RUN"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_rows(directory: Path, rows: list[dict], expected_names: set[str], label: str) -> None:
    require({row["file"] for row in rows} == expected_names, f"{label} file set mismatch")
    for row in rows:
        path = directory / row["file"]
        require(path.is_file(), f"missing {label} file {row['file']}")
        data = path.read_bytes()
        require(len(data) == row["bytes"] and sha256(data) == row["sha256"], f"{label} file mismatch: {row['file']}")


def verify_rule_parent(package_dir: Path) -> dict:
    manifest_bytes = (package_dir / "evidence_manifest.json").read_bytes()
    seal_bytes = (package_dir / "package_seal.json").read_bytes()
    manifest = json.loads(manifest_bytes)
    seal = json.loads(seal_bytes)
    require(sha256(manifest_bytes) == RULE_MANIFEST_SHA256, "reused rule manifest hash mismatch")
    require(sha256(seal_bytes) == RULE_SEAL_SHA256, "reused rule seal hash mismatch")
    require(manifest["package_id"] == RULE_PACKAGE_ID and seal["package_id"] == RULE_PACKAGE_ID, "reused rule package identity mismatch")
    for row in seal["files"]:
        data = (package_dir / row["file"]).read_bytes()
        require(len(data) == row["bytes"] and sha256(data) == row["sha256"], f"reused rule sealed payload mismatch: {row['file']}")
    return manifest


def main() -> None:
    package = Path(__file__).resolve().parent
    identities = json.loads((package / "SOURCE-IDENTITIES.json").read_text(encoding="utf-8"))
    manifest = json.loads((package / "evidence_manifest.json").read_text(encoding="utf-8"))
    seal = json.loads((package / "package_seal.json").read_text(encoding="utf-8"))
    evidence = json.loads((package / "FIRST-CONSUMER-EDGE.json").read_text(encoding="utf-8"))

    require(manifest["package_id"] == PACKAGE_ID and seal["package_id"] == PACKAGE_ID, "package id mismatch")
    require(manifest["status"] == EXPECTED_STATUS, "package status mismatch")
    require(manifest["runtime_execution"] == "NOT_RUN", "runtime must remain NOT_RUN")
    require(manifest["next_pcm_or_pts_consumer"] == "NOT_INSPECTED", "downstream scope must remain open")
    require(manifest["source_evidence"] == identities["sources"], "source identity list mismatch")

    payload_names = {"README.md", "FINDINGS.md", "FIRST-CONSUMER-EDGE.json", "SOURCE-IDENTITIES.json", "CHECK.py"}
    verify_rows(package, manifest["payloads"], payload_names, "payload")
    verify_rows(package, seal["files"], payload_names | {"evidence_manifest.json"}, "sealed")
    require(seal["file_count"] == len(payload_names) + 1, "seal file count mismatch")
    require(seal["seal_excluded_from_its_own_hash_list"] is True, "seal self-hash rule mismatch")

    source_bytes = {}
    for source in identities["sources"]:
        path = Path(source["absolute_path"])
        require(path.is_file(), f"missing source {source['name']}")
        data = path.read_bytes()
        require(len(data) == source["bytes"] and sha256(data) == source["sha256"], f"source hash mismatch: {source['name']}")
        source_bytes[source["kind"]] = data

    require(sha256(source_bytes["target_image"]) == DLL_SHA256, "target image hash mismatch")
    function_region = source_bytes["target_image"][CALLEE_FILE_OFFSET : CALLEE_FILE_OFFSET + CALLEE_BYTES]
    require(len(function_region) == CALLEE_BYTES and sha256(function_region) == CALLEE_SHA256, "FUN_1801844c0 function bytes mismatch")

    rule_package = Path(identities["rule_package_dir"])
    rule_manifest = verify_rule_parent(rule_package)
    require(rule_manifest["status"].startswith("STATIC_SAMPLE_CONVERSION_BOUND; ENCODE_RECORD_FLOW_BOUND"), "reused rule status mismatch")

    range_raw = source_bytes["range_consumer_raw"].decode("utf-8", errors="replace")
    copy_raw = source_bytes["range_boundary_copies_raw"].decode("utf-8", errors="replace")
    require("FUN_18013be90(param_2 + 0x18,param_3 + 7)" in copy_raw, "prepend vector copy into encode context missing")
    require("FUN_18013be90(param_2 + 0x60,param_3 + 10)" in copy_raw, "append vector copy into encode context missing")
    require("longlong * FUN_18013be90(longlong *param_1,longlong *param_2)" in copy_raw, "vector copy helper signature missing")
    require("uVar4 = lVar5 - lVar1 >> 3" in copy_raw and "FUN_18059a7e0(lVar2,lVar1,lVar5 - lVar1)" in copy_raw, "eight-byte vector copy stride missing")
    require("void FUN_18014e980(ulonglong *param_1,undefined8 *param_2)" in copy_raw, "qword vector append helper signature missing")
    require("*(undefined8 *)param_1[1] = *param_2" in copy_raw and "param_1[1] = param_1[1] + 8" in copy_raw, "qword append operation missing")

    consumer_log = source_bytes["consumer_decomp_log"].decode("utf-8", errors="replace")
    require("FUNCTION=FUN_1801844c0@1801844c0" in consumer_log and "DECOMPILE_STATUS=COMPLETE" in consumer_log, "first consumer Ghidra decompilation missing")
    require("FUN_1801844c0(ulonglong *param_1,longlong *param_2,longlong *param_3,longlong *param_4," in consumer_log and "longlong *param_6)" in consumer_log, "six-vector consumer prototype mismatch")
    require("param_2[1] - lVar1 >> 5 != param_3[1] - *param_3 >> 3" in consumer_log, "per-source vector-size check missing")
    require("uVar5 = *(undefined8 *)(*local_110 + uVar12 * 8)" in consumer_log and "uVar2 = *(undefined8 *)(*local_128 + uVar12 * 8)" in consumer_log, "prepend/append vector element reads missing")
    require("*(undefined8 *)(pcVar6 + 0x20) = local_78" in consumer_log and "*(undefined8 *)(pcVar6 + 0x38) = local_60" in consumer_log, "composite record field copies missing")
    require("makeCompositeMasterPrams: input vectors size mismatch" in consumer_log, "composite record builder role string missing")

    callsite_log = source_bytes["callsite_window_log"].decode("utf-8", errors="replace")
    for instruction in [
        "INSTRUCTION|0x175dc4|CALL|0x180133360",
        "INSTRUCTION|0x175dd4|CALL|0x180133360",
        "INSTRUCTION|0x175de4|CALL|0x180133360",
        "INSTRUCTION|0x175df9|MOV|qword ptr [RSP + 0x28], R14",
        "INSTRUCTION|0x175dfe|MOV|qword ptr [RSP + 0x20], RSI",
        "INSTRUCTION|0x175e03|MOV|R9, RDI",
        "INSTRUCTION|0x175e06|MOV|R8, RBX",
        "INSTRUCTION|0x175e09|MOV|RDX, RAX",
        "INSTRUCTION|0x175e10|CALL|0x1801844c0",
    ]:
        require(instruction in callsite_log, f"consumer callsite argument instruction missing: {instruction}")
    require("FIRST_CONSUMER_CALL=FUN_1801844c0|RVA=0x175e10" in callsite_log and "INSTRUCTION_COUNT_THROUGH_CALL=25" in callsite_log, "bounded callsite stop receipt missing")
    require("-readOnly" in source_bytes["consumer_runner"].decode("utf-8", errors="replace") and "-noanalysis" in source_bytes["consumer_runner"].decode("utf-8", errors="replace"), "Ghidra readonly/noanalysis boundary missing")
    require("-readOnly" in source_bytes["callsite_runner"].decode("utf-8", errors="replace") and "-noanalysis" in source_bytes["callsite_runner"].decode("utf-8", errors="replace"), "callsite Ghidra readonly/noanalysis boundary missing")

    require(evidence["first_typed_consumer"]["callee"] == "FUN_1801844c0", "consumer function mismatch")
    require(evidence["first_typed_consumer"]["output_mapping"]["prepend_output_qword_offset"] == "0x20", "prepend composite field mismatch")
    require(evidence["first_typed_consumer"]["output_mapping"]["append_output_qword_offset"] == "0x38", "append composite field mismatch")
    require(evidence["remaining_unknowns"]["actual_pcm_silence_materialization"] == "UNBOUND", "PCM effect must remain unbound")
    require(evidence["remaining_unknowns"]["native_pts_or_clock_effect"] == "UNBOUND", "PTS/clock effect must remain unbound")
    require(evidence["remaining_unknowns"]["first_downstream_consumer_of_composite_rows"] == "NOT_INSPECTED", "downstream scope mismatch")

    print("ATMOS_SILENCE_FIRST_CONSUMER_CHECK=PASS")
    print("EDGE=prepend_vector->param3/R8; append_vector->param6/stack; COMPOSITE_QWORDS=+0x20/+0x38; PCM_PTS_EFFECT=UNBOUND")


if __name__ == "__main__":
    main()
