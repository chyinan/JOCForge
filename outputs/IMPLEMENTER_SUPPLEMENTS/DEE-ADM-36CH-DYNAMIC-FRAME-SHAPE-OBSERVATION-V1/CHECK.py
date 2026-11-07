from __future__ import annotations
import hashlib, json, re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "DEE-ADM-36CH-DYNAMIC-FRAME-SHAPE-OBSERVATION-V1"
SOURCE_MANIFEST = "e96a393c1a0a4c3087890afc9cd5e8ad3b5bdd81a64c2f07c00c6359892f9923"
SOURCE_SEAL = "5ebfe35afeb73fc7981106f1630d9902845e9f2d037eff4ba9932c89c20ef1f6"
PAYLOADS = {"EVIDENCE.json", "INPUT-ONLY-VECTORS.json", "REPORT.md", "SOURCE-MANIFESTS.json"}

def require(value, message):
    if not value:
        raise RuntimeError(f"FAIL {PACKAGE_ID}: {message}")

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

manifest_path = ROOT / "MANIFEST.json"
seal_path = ROOT / "package_seal.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
seal = json.loads(seal_path.read_text(encoding="utf-8"))
evidence = json.loads((ROOT / "EVIDENCE.json").read_text(encoding="utf-8"))
require(manifest.get("package_id") == PACKAGE_ID, "package id")
require(manifest.get("source_manifest_sha256") == SOURCE_MANIFEST, "source manifest")
require(manifest.get("source_package_seal_sha256") == SOURCE_SEAL, "source seal")
require(seal.get("manifest_sha256") == digest(manifest_path), "manifest hash")
require({p.get("name") for p in manifest.get("payloads", [])} == PAYLOADS | {"CHECK.py"}, "payload inventory")
for row in manifest["payloads"]:
    path = ROOT / row["name"]
    require(path.is_file() and path.stat().st_size == row["bytes"], "payload size/missing")
    require(digest(path) == row["sha256"], "payload digest")
require(evidence["attempt_id"] == "attempt-05", "attempt identity")
require(evidence["native_count_domains"]["fmt_channel_count"]["values"] == [36], "fmt count")
require(evidence["native_count_domains"]["chna_physical_entry_count"]["values"] == [36], "CHNA count")
require(evidence["native_count_domains"]["k_descriptor_count"]["values"] == [36] * 4, "K counts")
require(evidence["generation_lineage"]["u_to_k"].startswith("UNBOUND"), "U/K must remain unbound")
require(evidence["generation_lineage"]["k_to_t_j_b_to_d"]["status"] == "JOINED_FOR_TGEN_1", "K/B/D join")
require(evidence["first_nonempty_frame_shape"]["actual_channels"] == 36, "actual N")
require(evidence["first_nonempty_frame_shape"]["processor_expected_channels"] == 0, "first transition")
require(evidence["first_nonempty_frame_shape"]["next_nonempty_call_expected_channels"] == 36, "next expected N")
require(evidence["model_and_later_stages"]["six_plane_value_comparison"] == "NOT_RUN", "audio comparison must remain unrun")
require(evidence["limits"]["general_channel_count_range"] == "NOT_ESTABLISHED", "range overclaim")
require(evidence["observer_scope"]["raw_pcm_or_adm_essence_saved"] is False, "PCM persistence")
joined = json.dumps(evidence, ensure_ascii=True) + "\n" + (ROOT / "REPORT.md").read_text(encoding="utf-8")
require(re.search(r"[A-Za-z]:\\", joined) is None, "absolute path leak")
require(re.search(r"0x[0-9a-fA-F]{6,}", joined) is None, "raw address leak")
require("private key" not in joined.lower() and "license.lic" not in joined.lower(), "private material")
require(manifest.get("contains_private_key_or_license") is False, "secret flag")
require(manifest.get("contains_pcm_or_media") is False, "media flag")
require(manifest.get("contains_native_source_or_decompilation") is False, "code flag")
print(f"PASS package={PACKAGE_ID} attempt=05 N36 K_to_B_to_D=BOUND U_to_K=UNBOUND")
