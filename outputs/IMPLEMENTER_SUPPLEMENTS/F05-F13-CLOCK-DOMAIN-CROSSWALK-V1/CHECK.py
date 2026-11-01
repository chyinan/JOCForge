import hashlib
import json
import math
import sys
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
PAYLOADS = {"CHECK.py", "CLOCK-VECTORS.json", "CONTRACT.md", "SOURCE-MANIFESTS.json", "WASH-VERIFICATION.json"}
PACKAGE_ID = "F05-F13-CLOCK-DOMAIN-CROSSWALK-V1"
INCOMING_MANIFEST = "02479cf47526b5fd16f3160bc3626631106683c0c212ff75678c79d0357c1a27"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(name):
    return json.loads((PACKAGE / name).read_text(encoding="utf-8"))


def build_manifest():
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()} - {"MANIFEST.json"}
    require(actual == PAYLOADS, f"payload inventory mismatch: {sorted(actual ^ PAYLOADS)}")
    files = [{"file": name, "bytes": (PACKAGE / name).stat().st_size, "sha256": sha256(PACKAGE / name)}
             for name in sorted(PAYLOADS)]
    manifest = {
        "schema": "jocforge-washed-implementer-supplement.v1",
        "package_id": PACKAGE_ID,
        "status": "APPROVED_PARENT_CLOCKS_BOUND; PROVIDER_CACHE_INDEX_SEMANTICS_UNSPECIFIED",
        "implementation_ready": False,
        "source_reverse_manifest_sha256": INCOMING_MANIFEST,
        "files": files,
    }
    (PACKAGE / "MANIFEST.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def verify():
    expected = PAYLOADS | {"MANIFEST.json"}
    actual = {path.name for path in PACKAGE.iterdir() if path.is_file()}
    require(actual == expected, f"file inventory mismatch: {sorted(actual ^ expected)}")
    manifest = read_json("MANIFEST.json")
    require(manifest["schema"] == "jocforge-washed-implementer-supplement.v1"
            and manifest["package_id"] == PACKAGE_ID
            and manifest["implementation_ready"] is False
            and manifest["source_reverse_manifest_sha256"] == INCOMING_MANIFEST,
            "manifest identity/readiness mismatch")
    rows = {row["file"]: row for row in manifest["files"]}
    require(len(rows) == len(manifest["files"]) and set(rows) == PAYLOADS, "manifest payload inventory mismatch")
    for name, row in rows.items():
        path = PACKAGE / name
        require(path.stat().st_size == row["bytes"] and sha256(path) == row["sha256"],
                f"payload hash mismatch: {name}")

    clocks = read_json("CLOCK-VECTORS.json")
    primary = clocks["primary_example"]
    n, h, q = primary["N"], primary["H"], primary["Q"]
    require(math.ceil(n / h) == q == 4247, "normal callback count mismatch")
    require(n - (q - 1) * h == primary["last_normal_real_samples"] == 1050,
            "last-normal real-sample count mismatch")
    require(h - primary["last_normal_real_samples"] == primary["last_normal_zero_padding"] == 486,
            "last-normal padding mismatch")
    counts = clocks["counts"]
    require(counts == {"main_callbacks": q + 4, "codec_sources": q + 2, "joc_dispatches": q + 2,
                       "published_aus": q, "meter_callbacks": q + 3},
            "primary count totals mismatch")
    first, cached_first, cached_last, meter = clocks["vectors"]
    require(first["g"] == 2 and first["s"] == first["g"] - 2 and first["d"] == first["s"] + 1,
            "first source/dispatch projection mismatch")
    require(cached_first["g"] == q and cached_first["s"] == q - 2 and cached_first["d"] == q - 1
            and cached_first["payload_origin"]["ordinal"] == q - 1,
            "first cached callback clock mismatch")
    require(cached_last["g"] == q + 3 and cached_last["s"] == q + 1 and cached_last["d"] == q + 2
            and cached_last["payload_origin"]["ordinal"] == q - 1,
            "last cached callback clock mismatch")
    require(cached_first["provider_embedded_indices"] == {"F01E": "UNSPECIFIED", "F03W": "UNSPECIFIED", "F04": "UNSPECIFIED"}
            and cached_last["provider_embedded_indices"] == cached_first["provider_embedded_indices"],
            "provider cached-index semantics were promoted")
    require(meter["m"] == q and meter["source"] is None and meter["joc_dispatch"] is None
            and meter["global_phase_generation_per_drain"] == "UNSPECIFIED"
            and meter["post_prepass_meter_ready_parent_commit_delta"] == 1,
            "meter clock/parent-commit boundary mismatch")
    require(clocks["expected_attempt07_compare_values"] == "NOT_CAPTURED",
            "attempt mismatch values were fabricated")

    sources = read_json("SOURCE-MANIFESTS.json")
    require(sources["incoming_package"]["manifest_sha256"] == INCOMING_MANIFEST
            and sources["incoming_package"]["checker_normal"] == "PASS"
            and sources["incoming_package"]["checker_python_optimized"] == "PASS"
            and sources["incoming_package"]["referenced_source_hashes_checked"] == 16
            and sources["incoming_package"]["source_hash_mismatches"] == 0,
            "incoming package/source audit mismatch")
    verification = read_json("WASH-VERIFICATION.json")
    require(verification["incoming_manifest_sha256"] == INCOMING_MANIFEST,
            "Wash verification provenance mismatch")
    print(json.dumps({"status": "VALID_PARTIAL", "package_id": PACKAGE_ID,
                      "manifest_sha256": sha256(PACKAGE / "MANIFEST.json"),
                      "Q": q, "main_callbacks": counts["main_callbacks"],
                      "codec_sources": counts["codec_sources"], "meter_callbacks": counts["meter_callbacks"],
                      "provider_cached_indices": "UNSPECIFIED"}, sort_keys=True))


if __name__ == "__main__":
    if sys.argv[1:] == ["--build-manifest"]:
        build_manifest()
    else:
        verify()
