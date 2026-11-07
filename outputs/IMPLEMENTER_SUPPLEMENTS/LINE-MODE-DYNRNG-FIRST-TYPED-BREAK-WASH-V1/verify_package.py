import hashlib
import json
from pathlib import Path


PACKAGE = Path(__file__).resolve().parent
IMPLEMENTER_SUPPLEMENT_ROOT = Path(r"D:\Programs\JOCForge-isolated\outputs\IMPLEMENTER_SUPPLEMENTS")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def manifest_items(manifest: dict) -> list[dict]:
    items = manifest.get("files", manifest.get("artifacts"))
    if items is None and isinstance(manifest.get("payloads"), dict):
        items = [dict(name=name, **details) for name, details in manifest["payloads"].items()]
    require(isinstance(items, list), "source manifest has no file list")
    return items


def verify_source_package(ref: dict) -> None:
    root = (IMPLEMENTER_SUPPLEMENT_ROOT / ref["directory"]).resolve()
    require(root.is_relative_to(IMPLEMENTER_SUPPLEMENT_ROOT), "source package path escaped root")
    manifest_bytes = (root / "MANIFEST.json").read_bytes()
    manifest_sha = digest(manifest_bytes)
    require(manifest_sha == ref["manifest_sha256"], f"source manifest changed: {ref['package_id']}")
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    if "package_id" in manifest:
        require(manifest["package_id"] == ref["package_id"], f"source package id mismatch: {ref['package_id']}")

    seal_path = root / "package_seal.json"
    require(seal_path.is_file(), f"source package seal missing: {ref['package_id']}")
    seal = read_json(seal_path)
    require(seal.get("manifest_sha256") == manifest_sha, f"source package seal mismatch: {ref['package_id']}")

    for item in manifest_items(manifest):
        relative = item.get("name", item.get("path"))
        require(isinstance(relative, str), f"source artifact path missing: {ref['package_id']}")
        payload_path = (root / relative).resolve()
        require(payload_path.is_relative_to(root), f"source artifact escaped package: {relative}")
        payload = payload_path.read_bytes()
        expected_sha = item.get("sha256")
        require(expected_sha == digest(payload), f"source artifact digest mismatch: {ref['package_id']}:{relative}")
        if "bytes" in item:
            require(item["bytes"] == len(payload), f"source artifact size mismatch: {ref['package_id']}:{relative}")


def main() -> None:
    manifest_bytes = (PACKAGE / "MANIFEST.json").read_bytes()
    receipt = (PACKAGE / "MANIFEST.sha256").read_text(encoding="ascii").split()
    require(receipt[0] == digest(manifest_bytes), "Wash manifest receipt mismatch")
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    require(manifest.get("package_id") == "LINE-MODE-DYNRNG-FIRST-TYPED-BREAK-WASH-V1", "wrong Wash package id")
    for item in manifest["artifacts"]:
        path = (PACKAGE / item["path"]).resolve()
        require(path.is_relative_to(PACKAGE), f"Wash artifact escaped package: {item['path']}")
        payload = path.read_bytes()
        require(len(payload) == item["bytes"], f"Wash artifact size mismatch: {item['path']}")
        require(digest(payload) == item["sha256"], f"Wash artifact digest mismatch: {item['path']}")

    refs = read_json(PACKAGE / "SOURCE-PACKAGES.json")["packages"]
    for ref in refs:
        verify_source_package(ref)

    evidence = read_json(PACKAGE / "EVIDENCE.json")
    require(evidence["first_typed_break"]["selected_zero_branch"]["row_owner_join_to_plus_0xA0"] == "UNBOUND", "row owner join was promoted")
    require(evidence["separate_native_core_curve_and_dynrng_evidence"]["xml_line_string_to_preset_selector"] == "UNBOUND", "curve selector edge was promoted")
    require(evidence["observations_reused"]["new_A_B_runs"] == 0, "A/B rerun was recorded")
    require(evidence["limits"]["DEE_run"] == "NOT_RUN", "DEE status changed")
    print(f"PASS: Wash package and {len(refs)} sealed source packages")


if __name__ == "__main__":
    main()
