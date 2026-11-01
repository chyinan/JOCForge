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
    sidecar_digest = sidecar_path.read_text(encoding="ascii").split()[0]
    require(sha256(manifest_path) == sidecar_digest, "manifest sidecar mismatch")
    require(manifest.get("schema") == "wash-static-trace-manifest.v1", "manifest schema mismatch")

    for artifact in manifest.get("artifacts", []):
        path = ROOT / artifact["name"]
        require(path.is_file(), f"artifact missing: {artifact['name']}")
        require(path.stat().st_size == artifact["bytes"], f"artifact size mismatch: {artifact['name']}")
        require(sha256(path) == artifact["sha256"], f"artifact hash mismatch: {artifact['name']}")

    evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
    check = json.loads((ROOT / "CHECK.json").read_text(encoding="utf-8"))
    require(evidence["identity"]["authorized_sample_id"] == "sample-6", "sample identity mismatch")
    require(evidence["identity"]["authorized_sample_sha256"] == "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1", "sample hash mismatch")
    require(evidence["static_range_path"]["first_consumer_role"] == "MULTI_SOURCE_IO_SET_BUILDER", "range consumer changed")
    require(evidence["spatial_time_candidate"]["query_type"] == "IEEE754_BINARY32", "spatial query type changed")
    require(evidence["same_call_join"]["range_io_set_to_spatial_fit"] == "UNBOUND", "spatial join overclaimed")
    require(evidence["same_call_join"]["reader_cursor_to_spatial_fit"] == "UNBOUND", "reader join overclaimed")
    require(evidence["execution"]["new_native_run"] == "NOT_RUN", "new native run was claimed")
    require(evidence["execution"]["model_replay"] == "NOT_RUN", "model replay was claimed")
    require(evidence["execution"]["gate1_closed"] is False, "Gate 1 was overclaimed")
    require(evidence["delivery"]["implementation_agent_contacted"] is False, "implementation contact was claimed")
    require(check["checks"]["reader_or_absolute_sample_to_spatial_fit"] == "UNBOUND", "check record mismatch")
    require(check["checks"]["gate1_closed"] is False, "check Gate 1 mismatch")

    forbidden = ("D:\\", "E:\\", "0x180", ".dll", "license.lic")
    for name in ("README.md", "EVIDENCE.json", "CHECK.json"):
        body = (ROOT / name).read_text(encoding="utf-8").lower()
        for token in forbidden:
            require(token.lower() not in body, f"sanitized artifact contains forbidden token: {token}")

    print("MANIFEST=PASS")
    print("RANGE_TO_IO_SET=STATICALLY_BOUND")
    print("SPATIAL_FIT_JOIN=UNBOUND")
    print("NEW_NATIVE_RUN=NOT_RUN")
    print("GATE1_CLOSED=NO")


if __name__ == "__main__":
    main()
