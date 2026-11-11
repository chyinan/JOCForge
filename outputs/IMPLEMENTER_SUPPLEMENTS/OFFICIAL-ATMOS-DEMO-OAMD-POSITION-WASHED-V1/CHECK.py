import hashlib
import json
from pathlib import Path

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

root = Path(__file__).resolve().parent
manifest_bytes = (root / "MANIFEST.json").read_bytes()
manifest = json.loads(manifest_bytes)
seal = json.loads((root / "PACKAGE-SEAL.json").read_bytes())
require(manifest["package_id"] == "OFFICIAL-ATMOS-DEMO-OAMD-POSITION-WASHED-V1", "package id mismatch")
require(manifest["production_allowed"] is False, "production gate mismatch")
require(seal["manifest_sha256"] == hashlib.sha256(manifest_bytes).hexdigest(), "manifest digest mismatch")
require(seal["file_paths"] == [item["path"] for item in manifest["files"]], "seal path list mismatch")
for item in manifest["files"]:
    path = root / item["path"]
    require(path.is_file(), f"missing file: {item['path']}")
    require(path.stat().st_size == item["bytes"], f"size mismatch: {item['path']}")
    require(sha(path) == item["sha256"], f"hash mismatch: {item['path']}")
summary = json.loads((root / "OAMD-POSITION-OBSERVATION.json").read_bytes())
require(summary["source"]["sha256"] == "f976aba7a4750be80b4373a5d7fabf2ad101e0c227fc4bd77716b76879d546cd", "source identity mismatch")
require(summary["oamd"]["object_slots"]["dynamic"] == 15, "dynamic object count mismatch")
require(summary["dynamic_position"]["explicit_absolute_position_fields_total"] == 345495, "position count mismatch")
require(summary["dynamic_position"]["z_code_distribution"] == {"code_0": 207297, "code_15_positive": 138198}, "Z distribution mismatch")
require(summary["rate_comparison"]["target_output_validation"] == "NOT_EVALUATED", "target status mismatch")
require(summary["validation_disposition"]["etsi_strict"] == "FAILS_TABLE56_FOR_THIS_SAMPLE", "Table 56 disposition mismatch")
receipt = json.loads((root / "SOURCE-RECEIPTS.json").read_bytes())
require(receipt["exclusions"]["mka_or_packets_included"] is False, "media inclusion boundary mismatch")
require(receipt["exclusions"]["oamd_payload_or_coordinate_samples_included"] is False, "payload inclusion boundary mismatch")
parent = root.parent / "OFFICIAL-ATMOS-DEMO-STRUCTURE-WASHED-V2"
require(sha(parent / "MANIFEST.json") == "c4104e674a9c3df06ead8f6741a7366e58068602789806d1a91d045c1dfd731a", "parent manifest mismatch")
require(sha(parent / "PACKAGE-SEAL.json") == "0fb4c7f8a7a49488a97586b7e3eb6febc0542cf0044569fe37e65dc07d85757d", "parent seal mismatch")
print("PASS official demo OAMD position summary; source, parent, field-domain, and NOT_EVALUATED boundaries bound")