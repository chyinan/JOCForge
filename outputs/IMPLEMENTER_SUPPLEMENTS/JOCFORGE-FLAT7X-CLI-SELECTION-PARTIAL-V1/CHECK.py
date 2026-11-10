from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent

def fail(message):
    raise SystemExit("FAIL " + message)

def main():
    manifest_path = ROOT / "MANIFEST.json"
    seal_path = ROOT / "PACKAGE-SEAL.json"
    if not manifest_path.is_file() or not seal_path.is_file():
        fail("manifest or seal missing")
    manifest_bytes = manifest_path.read_bytes()
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    seal = json.loads(seal_path.read_text(encoding="utf-8"))
    digest = hashlib.sha256(manifest_bytes).hexdigest()
    if manifest.get("package_id") != "JOCFORGE-FLAT7X-CLI-SELECTION-PARTIAL-V1":
        fail("package id mismatch")
    if seal.get("manifest_sha256") != digest:
        fail("seal manifest hash mismatch")
    if seal.get("package_id") != manifest.get("package_id"):
        fail("seal package id mismatch")
    files = manifest.get("files")
    if not isinstance(files, list) or not files:
        fail("file list invalid")
    seen = set()
    for item in files:
        name = item.get("path")
        if not isinstance(name, str) or name in seen or Path(name).name != name:
            fail("unsafe or duplicate path")
        seen.add(name)
        data = (ROOT / name).read_bytes()
        if len(data) != item.get("bytes"):
            fail("size mismatch: " + name)
        if hashlib.sha256(data).hexdigest() != item.get("sha256"):
            fail("hash mismatch: " + name)
    vectors = json.loads((ROOT / "INPUT-ONLY-VECTORS.json").read_text(encoding="utf-8"))
    if any(v.get("status") != "INPUT_ONLY_NOT_RUN" for v in vectors.get("vectors", [])):
        fail("input-only vector disposition mismatch")
    if seal.get("status") != "SEALED_WASH_SUPPLEMENT":
        fail("seal status mismatch")
    print("PASS Flat7x CLI selector scope, input-only vectors, payload hashes, and seal")

if __name__ == "__main__":
    main()
