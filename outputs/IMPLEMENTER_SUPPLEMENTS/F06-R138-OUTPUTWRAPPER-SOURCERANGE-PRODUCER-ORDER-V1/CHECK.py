from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


manifest_path = ROOT / "MANIFEST.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
require(sha256(manifest_path) == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
require(manifest.get("package_id") == "F06-R138-OUTPUTWRAPPER-SOURCERANGE-PRODUCER-ORDER-V1", "package id")
payload = manifest.get("files")
require(isinstance(payload, list) and len(payload) == 4, "payload inventory")
actual_names = {path.name for path in ROOT.iterdir() if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}}
require(actual_names == {entry.get("file") for entry in payload}, "payload file set")
for entry in payload:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"hash {entry['file']}")

seal = json.loads((ROOT / "package_seal.json").read_text(encoding="utf-8"))
sealed = seal.get("files")
require(isinstance(sealed, list) and seal.get("file_count") == len(sealed), "seal inventory")
require({entry.get("file") for entry in sealed} == actual_names | {"MANIFEST.json", "MANIFEST.sha256"}, "sealed file set")
for entry in sealed:
    path = ROOT / entry["file"]
    require(path.stat().st_size == entry["bytes"], f"sealed size {entry['file']}")
    require(sha256(path) == entry["sha256"], f"sealed hash {entry['file']}")

evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
vector = json.loads((ROOT / "INPUT-ONLY-VECTOR.json").read_text(encoding="utf-8"))
facts = evidence["bound_static_facts"]
require(facts["outputwrapper_member"] == "+0x1A0", "output range identity")
require(facts["source_registry_member"] == "+0x178", "registry identity")
require(facts["post_filter_order"] == "same as registry traversal order for matched rows", "local order rule")
require(evidence["unbound_edges"]["registry_equals_job_file_list"] == "UNBOUND", "job-list join remains open")
require(evidence["unbound_edges"]["registry_equals_adm_converter_channel_vector"] == "UNBOUND", "converter join remains open")
require(vector["vector_status"] == "NOT_PROVIDED" and vector["runtime_execution"] == "NOT_RUN", "no runtime vector claimed")
for key in ("dee_run", "media_or_pcm_read", "key_or_license_read", "implementation_contact", "raw_source_or_decompilation_included"):
    require(evidence["execution_boundary"][key] is False, f"boundary {key}")

print("PASS: hash-bound local filtered-order rule; job/ADM owner joins and runtime vector remain explicitly unbound/not run")
