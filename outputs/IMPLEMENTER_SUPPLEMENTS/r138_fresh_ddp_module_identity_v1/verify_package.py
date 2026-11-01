from __future__ import annotations
import hashlib, json
from pathlib import Path
def require(ok, message):
    if not ok:
        raise SystemExit("FAIL: " + message)
def sha(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1048576), b""):
            h.update(block)
    return h.hexdigest()
def main():
    p = Path(__file__).resolve().parent
    mp = p / "MANIFEST.json"
    m = json.loads(mp.read_text(encoding="utf-8"))
    e = json.loads((p / "EVIDENCE.json").read_text(encoding="utf-8"))
    require(m.get("package_id") == "r138_fresh_ddp_module_identity_v1", "package id")
    require(m.get("implementation_delivery") == "PROHIBITED", "delivery boundary")
    require(e.get("status") == "FRESH_PROCESS_MODULE_SHA_BOUND_HISTORICAL_RUNS_UNBOUND", "status")
    require(e.get("input", {}).get("authorized_input_sha256") ==
            "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1", "input hash")
    require(e.get("input", {}).get("task_alias_sha256") == e["input"]["authorized_input_sha256"],
            "alias identity")
    require(e.get("loaded_module", {}).get("resolved_file_sha256") ==
            "3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2", "module hash")
    require(e.get("loaded_module", {}).get("same_process_pid") == e.get("process", {}).get("pid"),
            "process/module join")
    require(e.get("historical_runs") == {"Run145": "UNBOUND / NOT_EVALUATED",
                                         "Run147": "UNBOUND / NOT_EVALUATED"},
            "historical run boundary")
    require(e.get("execution", {}).get("temporary_ec3_read_or_hashed") is False and
            e.get("execution", {}).get("temporary_ec3_deleted") is True, "media boundary")
    entries = m.get("payloads")
    require(isinstance(entries, list) and
            {x.get("name") for x in entries} == {"README.md", "EVIDENCE.json", "MATRIX.json", "verify_package.py"},
            "payload inventory")
    for item in entries:
        name = item["name"]
        require(Path(name).name == name, "flat payload")
        path = p / name
        require(path.is_file() and path.stat().st_size == item.get("bytes") and
                sha(path) == item.get("sha256"), "payload hash")
    seal = (p / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha(mp) == seal[0],
            "manifest seal")
    print("PASS: fresh R138 module identity scope and hashes")
if __name__ == "__main__":
    main()
