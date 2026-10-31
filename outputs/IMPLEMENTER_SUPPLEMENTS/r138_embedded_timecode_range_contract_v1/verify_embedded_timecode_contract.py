import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main() -> None:
    manifest_path = ROOT / "MANIFEST.json"
    sidecar_path = ROOT / "MANIFEST.sha256"
    require(manifest_path.is_file(), "manifest missing")
    require(sidecar_path.is_file(), "manifest sidecar missing")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    require(sha256(manifest_path) == sidecar_path.read_text(encoding="ascii").split()[0], "manifest sidecar mismatch")
    require(manifest.get("schema") == "implementation-safe-contract-manifest.v1", "manifest schema mismatch")
    for artifact in manifest.get("artifacts", []):
        path = ROOT / artifact["name"]
        require(path.is_file(), f"artifact missing: {artifact['name']}")
        require(path.stat().st_size == artifact["bytes"], f"artifact size mismatch: {artifact['name']}")
        require(sha256(path) == artifact["sha256"], f"artifact hash mismatch: {artifact['name']}")

    contract = json.loads((ROOT / "CONTRACT.json").read_text(encoding="utf-8"))
    check = json.loads((ROOT / "CHECK.json").read_text(encoding="utf-8"))
    require(contract["evidence_identity"]["r138_configuration_sha256"] == "fa65143f0bd7de52b36367137e61d681baa4c17824aded10be26317e7d1018a4", "R138 configuration hash mismatch")
    require(contract["r138_selected_profile"]["time_base"] == "embedded_timecode", "R138 route mismatch")
    require(contract["r138_selected_profile"]["start_token"] == "86640", "start token mismatch")
    require(contract["r138_selected_profile"]["end_token"] == "86664", "end token mismatch")
    require(contract["r138_selected_profile"]["resolved_timecode_frame_rate"] == "NOT_CAPTURED", "FPS overclaimed")
    require(contract["conditional_branch_separation"]["applicable_to_r138_selected_route"] is False, "file-position branch promoted")
    require(contract["range_and_spatial_domains"]["range_to_spatial_same_time_domain"] == "UNBOUND", "spatial join overclaimed")
    require(contract["selected_call_position"]["status"] == "DERIVED_ONLY", "derived position overclaimed")
    require(contract["execution_status"]["new_native_job"] == "NOT_RUN", "native execution claimed")
    require(contract["execution_status"]["gate1_closed"] is False, "Gate 1 overclaimed")
    require(contract["delivery"]["implementation_agent_contacted"] is False, "implementation contact claimed")
    require(check["checks"]["timecode_frame_rate"] == "NOT_CAPTURED; NO FPS INFERENCE", "check FPS mismatch")

    forbidden = ("D:\\", "E:\\", "0x", ".dll", "license.lic")
    for name in ("README.md", "CONTRACT.json", "CHECK.json"):
        body = (ROOT / name).read_text(encoding="utf-8").lower()
        for token in forbidden:
            require(token.lower() not in body, f"sanitized contract contains forbidden content: {token}")

    print("MANIFEST=PASS")
    print("R138_TIME_BASE=EMBEDDED_TIMECODE")
    print("TIME_CODE_RATE=NOT_CAPTURED")
    print("FILE_POSITION_BRANCH=CONDITIONAL_ONLY")
    print("SPATIAL_TIME_DOMAIN_JOIN=UNBOUND")
    print("GATE1_CLOSED=NO")


if __name__ == "__main__":
    main()
