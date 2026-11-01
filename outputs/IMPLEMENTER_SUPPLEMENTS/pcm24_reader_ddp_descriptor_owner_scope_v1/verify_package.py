import hashlib
import json
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit("FAIL: " + message)

def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()

def main():
    root = Path(__file__).resolve().parent
    manifest_path = root / "MANIFEST.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    evidence = json.loads((root / "EVIDENCE.json").read_text(encoding="utf-8"))
    opened = json.loads((root / "OPEN-EDGES.json").read_text(encoding="utf-8"))
    require(manifest["package_id"] == "pcm24_reader_ddp_descriptor_owner_scope_v1", "package id")
    require(manifest["implementation_delivery"] == "ALLOWED_WITHIN_DECLARED_SCOPE_ONLY", "delivery scope")
    require(evidence["status"] == manifest["status"], "status binding")
    require(evidence["static_ddp_reader"]["descriptor_writes"]["offset0"] == "same DDP internal row-table value after read call", "descriptor offset 0")
    require(evidence["worker_and_consumer"]["same_local_descriptor_passed_to_source_then_process"] is True, "worker descriptor flow")
    require(evidence["worker_and_consumer"]["rule"] == "source_table[i] -> internal_row[i], frame_count * 4 bytes", "row copy rule")
    require(evidence["selected_reader_boundary"]["runtime_breakpoint"] == "NOT_HIT_IN_RUN148", "selected reader boundary")
    require(evidence["selected_reader_boundary"]["selected_reader_owner_and_generation"] == "UNBOUND/NO", "generation boundary")
    require(evidence["run148_context"]["output_bitstream_parsed"] is False, "output parse status")
    require(evidence["run148_context"]["playback"] == "NOT_RUN", "playback status")
    require(evidence["run148_context"]["license_contents_read"] is False, "license boundary")
    require(evidence["exclusions"]["raw_pointer_tokens_copied"] is False, "pointer-token exclusion")
    require(opened["package_id"] == manifest["package_id"], "open-edge package id")
    require({entry["name"] for entry in manifest["payloads"]} == {"README.md", "EVIDENCE.json", "OPEN-EDGES.json", "SOURCE-BINDING.json", "verify_package.py"}, "payload set")
    for entry in manifest["payloads"]:
        path = root / entry["name"]
        require(path.is_file(), "missing payload " + entry["name"])
        require(path.stat().st_size == entry["bytes"], "payload size " + entry["name"])
        require(sha256(path) == entry["sha256"], "payload hash " + entry["name"])
    seal = (root / "MANIFEST.sha256").read_text(encoding="ascii").strip().split()
    require(len(seal) == 2 and seal[1] == "MANIFEST.json" and sha256(manifest_path) == seal[0], "manifest seal")
    print("PASS: DDP internal descriptor bridge; selected reader owner and generation remain open")

if __name__ == "__main__":
    main()
