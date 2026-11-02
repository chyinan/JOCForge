from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE_ID = "SAMPLE6-AU-ALIGNED-DEE-CORE-PCM-COMPARISON-V2"
SOURCE_MANIFEST = "ade02cf02c9e0d7f1947148a9b188accad24900e649a4e67999c9c120fb96a49"
STATUS = "AU_ALIGNED_4S_CORE_PCM_MISMATCH; PER_CHANNEL_RESIDUALS_BOUND; CAUSE_UNBOUND"
EXPECTED = [
    (0, 0.0009079803907528591, 0.009047797791627243, 0.30480920523405075),
    (1, 0.0008499757858610802, 0.008921435265031322, 0.30366067588329315),
    (2, 0.00041541492581204103, 0.0037042021226497755, 0.1799558358361537),
    (3, 0.004220327010109068, 0.005479858820295154, 0.018465762957930565),
    (4, 0.0006096387851025907, 0.004608154658242228, 0.2434435160830617),
    (5, 0.0005502590218270345, 0.004344961223019292, 0.21280011942144483),
]


def require(condition: bool, label: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL {label}")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((ROOT / name).read_text(encoding="utf-8"))


def main() -> None:
    manifest = load("MANIFEST.json")
    manifest_hash = sha256(ROOT / "MANIFEST.json")
    require(manifest_hash == (ROOT / "MANIFEST.sha256").read_text(encoding="ascii").split()[0], "manifest hash")
    require(manifest["package_id"] == PACKAGE_ID and manifest["status"] == STATUS, "manifest id/status")
    require(manifest["source_reverse_manifest_sha256"] == SOURCE_MANIFEST, "source manifest hash")
    entries = {entry["file"]: entry for entry in manifest["files"]}
    actual = {
        path.name for path in ROOT.iterdir()
        if path.is_file() and path.name not in {"MANIFEST.json", "MANIFEST.sha256", "package_seal.json"}
    }
    require(actual == set(entries), "payload inventory")
    for name, entry in entries.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"size {name}")
        require(sha256(path) == entry["sha256"], f"hash {name}")
    seal = load("package_seal.json")
    sealed = {entry["file"]: entry for entry in seal["files"]}
    require(seal["package_id"] == PACKAGE_ID, "seal id")
    require(set(sealed) == set(entries) | {"MANIFEST.json", "MANIFEST.sha256"}, "seal inventory")
    for name, entry in sealed.items():
        path = ROOT / name
        require(path.stat().st_size == entry["bytes"], f"sealed size {name}")
        require(sha256(path) == entry["sha256"], f"sealed hash {name}")

    evidence = load("EVIDENCE.json")
    require(evidence["status"] == STATUS, "evidence status")
    require(evidence["window"]["sample_frames_per_channel"] == 192000, "window size")
    require(evidence["window"]["access_unit_interval"] == [2750, 2875], "AU interval")
    require(evidence["dee_reference"]["pcm_hash_provenance"] == "SOURCE_PACKAGE_ATTESTED_NOT_RECOMPUTED_BY_WASH", "DEE hash provenance")
    require(evidence["rust_reference"]["pcm_hash_provenance"] == "INDEPENDENTLY_RECOMPUTED_BY_WASH", "Rust hash provenance")
    observed = evidence["comparison"]["per_channel"]
    require(len(observed) == len(EXPECTED), "per-channel count")
    for item, expected in zip(observed, EXPECTED):
        index, mae, rmse, peak = expected
        require(item["channel_index"] == index, f"channel index {index}")
        require(item["mean_abs_error_f32"] == mae, f"MAE channel {index}")
        require(item["rmse_f32"] == rmse, f"RMSE channel {index}")
        require(item["max_abs_error_f32"] == peak, f"peak channel {index}")
    require(evidence["comparison"]["per_channel_correlation"] == "NOT_CAPTURED", "correlation boundary")
    require(evidence["comparison"]["per_channel_best_gain"] == "NOT_CAPTURED", "gain boundary")
    require(evidence["comparison"]["per_channel_best_lag"] == "NOT_CAPTURED", "lag boundary")
    require(evidence["comparison"]["cause"] == "UNBOUND", "cause boundary")
    require(evidence["scope"]["pcm_persisted"] is False, "PCM persistence")
    require(evidence["scope"]["atmos_spatial_comparison"] == "NOT_RUN", "spatial boundary")
    print(f"PASS package={PACKAGE_ID}")
    print(f"PASS manifest_sha256={manifest_hash}")
    print("PASS six per-channel residual records, source/hash provenance, and NOT_CAPTURED/UNBOUND boundaries")


if __name__ == "__main__":
    main()
