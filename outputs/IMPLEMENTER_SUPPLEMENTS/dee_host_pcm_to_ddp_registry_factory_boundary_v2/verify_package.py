from __future__ import annotations
import hashlib, json
from pathlib import Path
def require(ok, message):
    if not ok:
        raise SystemExit("FAIL: " + message)
def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1048576), b""):
            digest.update(block)
    return digest.hexdigest()
def main():
    package = Path(__file__).resolve().parent
    manifest_path = package / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    evidence = json.loads((package / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(manifest.get("implementation_delivery") == "PROHIBITED", "delivery boundary")
    require(evidence.get("implementation_effect") in
            {"NONE_AUDIO_ALGORITHM_RULE", "INTEGRATION_BOUNDARY_ONLY_NO_AUDIO_ALGORITHM_RULE"},
            "scope")
    if manifest["package_id"] == "r4_run_module_identity_boundary_v1":
        expected = {"README.md", "EVIDENCE.json", "RUN-MATRIX.json", "verify_package.py"}
    elif manifest["package_id"] == "dee_host_pcm_to_ddp_registry_factory_boundary_v2":
        expected = {"README.md", "EVIDENCE.json", "NEXT-EDGE.json", "verify_package.py"}
    elif manifest["package_id"] == "run147_payload11_presence_crosscheck_v1":
        expected = {"README.md", "EVIDENCE.json", "CURSOR-OBSERVATION.json", "verify_package.py"}
    else:
        expected = {"README.md", "EVIDENCE.json", "CURSOR-OBSERVATION.json", "verify_package.py"}
    payloads = manifest.get("payloads")
    require(isinstance(payloads, list) and {item.get("name") for item in payloads} == expected,
            "payload inventory")
    for item in payloads:
        name = item["name"]
        require(Path(name).name == name, "flat payload path")
        path = package / name
        require(path.is_file(), "payload exists")
        require(path.stat().st_size == item.get("bytes") and sha(path) == item.get("sha256"),
                "payload hash/size")
    seal = (package / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha(manifest_path) == seal[0],
            "manifest seal")
    upstreams = evidence.get("upstream_packages", [evidence.get("upstream_package")])
    require(all(item and item["checker_normal"] == "PASS" and
                item["checker_python_optimized"] == "PASS" for item in upstreams),
            "upstream checks")
    if manifest["package_id"] == "r4_run_module_identity_boundary_v1":
        rows = evidence["runs"]
        require([row["run_id"] for row in rows] == ["Run145", "Run147"], "run inventory")
        require(all(row["identity"] == "UNBOUND" and
                    row["target_module_sha_match"] == "NOT_EVALUATED" and
                    row["output_basename_matches_openjoc_input"] and
                    row["dee_encoder_completion_logged"] and
                    not row["loaded_module_receipt_present"] for row in rows), "identity state")
    elif manifest["package_id"] == "dee_host_pcm_to_ddp_registry_factory_boundary_v2":
        require(evidence["host_flow"]["registry_to_config"] == "BOUND" and
                evidence["host_flow"]["config_to_processing_stage"] == "BOUND", "host flow")
        require(evidence["unbound_edges"]["per_entry_callback_writer"] == "UNBOUND/NO" and
                evidence["unbound_edges"]["pcm_to_ddp_callback_target"] == "UNBOUND/NO" and
                evidence["unbound_edges"]["descriptor_offset_0"] == "UNBOUND/NO",
                "unbound edge boundary")
        require(evidence["execution"]["new_dee_job"] == "NOT_RUN" and
                evidence["execution"]["raw_pcm_read"] == "NOT_RUN", "execution boundary")
    elif manifest["package_id"] == "run147_payload11_presence_crosscheck_v1":
        observation = evidence["observation"]
        require(observation["payload_id"] == 11 and
                observation["group_id_present"] ==
                {"configuration_relative_bit_offset": 2, "status": "BIT_ONE"}, "Run147 cursor result")
        require(observation["absolute_au0_bit_offset"] == "NOT_CAPTURED" and
                observation["payload_body_read"] is False and observation["pcm_read"] is False,
                "Run147 scope")
        require(evidence["run145_comparison"]["group_id_present"] == "BIT_ONE", "cross-run value")
        require(evidence["run145_comparison"]["target_module_join"] == "UNBOUND",
                "cross-run identity boundary")
    else:
        observation = evidence["observation"]
        if "run145" in observation:
            require(observation["run145"]["payload_id"] == 11 and
                    observation["run145"]["group_id_present"] ==
                    {"absolute_file_bit_offset": 9156, "status": "BIT_ONE"}, "Run145 cursor result")
            require(observation["run147"]["payload_id"] == 11 and
                    observation["run147"]["group_id_present"] ==
                    {"configuration_relative_bit_offset": 2, "status": "BIT_ONE"} and
                    observation["run147"]["absolute_au0_bit_offset"] == "NOT_CAPTURED",
                    "Run147 cursor result")
            require(all(row["payload_body_read"] is False and
                        row["payload_body_saved"] is False and row["pcm_read"] is False
                        for row in observation.values()), "read boundary")
            require(evidence["static_comparison"]["same_loaded_module_run_join"] == "UNBOUND" and
                    evidence["execution"]["new_dee_job"] == "NOT_RUN", "join/execution boundary")
        else:
            require(observation["payload_id"] == 11 and
                    observation["group_id_present"] ==
                    {"bit_offset": 9156, "width": 1, "status": "BIT_ONE"}, "cursor result")
            require(observation["payload_body_read"] is False and
                    observation["payload_body_saved"] is False and observation["pcm_read"] is False,
                    "read boundary")
            require(evidence["static_comparison"]["same_loaded_module_run_join"] == "UNBOUND" and
                    evidence["reference_context"]["dee_run"] == "NOT_RUN", "join/execution boundary")
    print("PASS: washed reference evidence scope and hashes")
if __name__ == "__main__":
    main()
