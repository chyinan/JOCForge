import hashlib
import json
from pathlib import Path


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


root = Path(__file__).resolve().parent
manifest_bytes = (root / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
sidecar = (root / "MANIFEST.sha256").read_text(encoding="ascii").split()
manifest_sha256 = hashlib.sha256(manifest_bytes).hexdigest()
require(sidecar == [manifest_sha256, "MANIFEST.json"], "manifest sidecar mismatch")
require(
    manifest.get("package_id") == "METER-AUTHORIZED-ADM-Q3-PREPASS-RESULT-V1"
    and manifest.get("status") == "LOCAL_REAL_INPUT_Q3_PASS_PARENT_READY_NOT_RUN",
    "package identity/status mismatch",
)

expected_paths = set()
for record in manifest.get("files", []):
    relative = record["path"]
    expected_paths.add(relative)
    payload = (root / relative).read_bytes()
    require(len(payload) == record["bytes"], f"byte-count mismatch: {relative}")
    require(hashlib.sha256(payload).hexdigest() == record["sha256"], f"sha256 mismatch: {relative}")
actual_paths = {
    path.name
    for path in root.iterdir()
    if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256"}
}
require(actual_paths == expected_paths, "package contains an unmanifested or missing file")

result = json.loads((root / "RUN-RESULT.json").read_bytes())
binding = json.loads((root / "SOURCE-BINDING.json").read_bytes())
require(result.get("run_status") == "PASS", "run did not pass")
require(
    result.get("evidence_tier") == "LOCAL_IGNORED_INTEGRATION_TEST_ON_USER_AUTHORIZED_REAL_INPUT",
    "evidence tier mismatch",
)
require(
    result["input"]["sha256"]
    == binding["input_sha256"]
    == "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1",
    "authorized input hash mismatch",
)
require(result["input"]["bytes"] == binding["input_bytes"] == 743696526, "input byte count mismatch")
require(
    result["input"]["path_recorded"] is False
    and result["input"]["pcm_persisted"] is False
    and binding["raw_pcm_in_package"] is False,
    "input path/PCM persistence boundary mismatch",
)
require(
    binding["partial_encoded_output_in_package"] is False
    and binding["private_key_material_in_package"] is False,
    "sanitized package boundary mismatch",
)
require(
    result["selected_configuration"]["job_xml_sha256"]
    == binding["selected_job_xml_sha256"]
    == "20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9",
    "selected job XML identity mismatch",
)
require(
    result["selected_configuration"]["matches_current_accepted_selection"] is True
    and binding["job_xml_is_current_accepted_selection"] is True,
    "job XML is not the current selected job",
)
require(
    result["test"]["passed"] == 1
    and result["test"]["failed"] == 0
    and result["test"]["duration_seconds"] == 3293.53,
    "test execution record mismatch",
)
schedule = result["prepass"]
require(
    schedule["normal_callbacks"] == 4247
    and schedule["fresh_zero_drains"] == 3
    and schedule["typed_meter_owner_receipts"] == 4250,
    "Q+3 schedule/receipt counts mismatch",
)
require(
    schedule["terminal_measurement_receipt_proposal"] is True
    and schedule["measurement_dependents_receipt_matches_terminal_receipt"] is True
    and schedule["id1_measurement_input_matches_prepared_source"] is True,
    "terminal measurement/dependent binding mismatch",
)
require(schedule["standalone_meter_root_unchanged"] is True, "parent root commit was incorrectly claimed")
require(all(result["not_run"].values()), "an unrun parent/product gate was reported as run")
summary = (root / "WASHED-SUMMARY.md").read_text(encoding="utf-8")
require("GTC READY AND PRODUCT OUTPUT NOT RUN" in summary, "summary overstates product readiness")
require("No input path, PCM, partial output" in summary, "summary omits the sanitized evidence boundary")
print("METER_AUTHORIZED_ADM_Q3_PACKAGE=PASS")
print(f"MANIFEST_SHA256={manifest_sha256}")
