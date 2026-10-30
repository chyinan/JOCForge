"""Build a searchable locator for washed implementer supplements.

Functional Core: normalize manifest entries and short, hash-checked summaries.
Imperative Shell: read package manifests and write one generated catalog file.
"""

from __future__ import annotations

import hashlib
import argparse
import json
import re
import sys
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
SUPPLEMENT_ROOT = REPOSITORY_ROOT / "outputs" / "IMPLEMENTER_SUPPLEMENTS"
CATALOG_PATH = SUPPLEMENT_ROOT / "CATALOG.md"
MANIFEST_NAMES = ("MANIFEST.json", "evidence_manifest.json", "manifest.json")
SUMMARY_NAMES = ("CONTRACT.md", "REPORT.md", "README.md", "SUMMARY.md")
STATUS_NAMES = ("CHECK.json", "VERIFICATION.json", "WASH-REVIEW.json", "PACKAGE-SEAL.json")
SUMMARY_LIMIT = 360
PAYLOAD_NAME_LIMIT = 8


def sha256_bytes(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def manifest_entries(manifest: dict) -> list[dict]:
    for key in ("files", "payloads"):
        entries = manifest.get(key)
        if isinstance(entries, list):
            return entries
    return []


def entry_name(entry: dict) -> str | None:
    for key in ("name", "file", "path"):
        value = entry.get(key)
        if isinstance(value, str):
            return value
    return None


def verified_payload(package_dir: Path, entry: dict) -> bytes | None:
    name = entry_name(entry)
    expected_hash = entry.get("sha256")
    expected_bytes = entry.get("bytes")
    if not name or not isinstance(expected_hash, str):
        return None

    path = (package_dir / name).resolve()
    try:
        path.relative_to(package_dir.resolve())
    except ValueError:
        return None
    if not path.is_file():
        return None

    content = path.read_bytes()
    if isinstance(expected_bytes, int) and len(content) != expected_bytes:
        return None
    if sha256_bytes(content) != expected_hash:
        return None
    return content


def scrub_summary(value: str) -> str:
    value = re.sub(r"```.*?```", " ", value, flags=re.DOTALL)
    value = re.sub(r"`([^`]*)`", r"\1", value)
    value = re.sub(r"(?i)\b[A-Z]:\\[^\s|]+", "[local path]", value)
    value = re.sub(r"(?i)\b0x[0-9a-f]{6,}\b", "[address]", value)
    value = re.sub(r"\s+", " ", value).strip()
    return value[:SUMMARY_LIMIT]


def first_summary_paragraph(content: bytes) -> str:
    text = content.decode("utf-8", errors="replace")
    paragraphs = re.split(r"\n\s*\n", text)
    selected = []
    for paragraph in paragraphs:
        lines = [line.strip() for line in paragraph.splitlines()]
        lines = [line for line in lines if line and not line.startswith(("#", ">", "|"))]
        if lines:
            selected.append(" ".join(lines))
            if len(" ".join(selected)) >= SUMMARY_LIMIT:
                break
    return scrub_summary(" ".join(selected))


def explicit_status(package_dir: Path, manifest: dict, entries: list[dict]) -> str:
    direct = manifest.get("status")
    if isinstance(direct, str) and direct:
        return f"manifest: {direct}"

    by_name = {entry_name(item): item for item in entries if entry_name(item)}
    for name in STATUS_NAMES:
        entry = by_name.get(name)
        if not entry:
            continue
        content = verified_payload(package_dir, entry)
        if content is None:
            return f"{name}: HASH_MISMATCH_OR_MISSING"
        try:
            status_doc = json.loads(content)
        except json.JSONDecodeError:
            continue
        for key in ("status", "overall_status", "implementation_readiness", "decision"):
            value = status_doc.get(key)
            if isinstance(value, str) and value:
                return f"{name}:{key}={value}"
    return "NOT_DECLARED"


def package_summary(package_dir: Path, manifest: dict, entries: list[dict]) -> str:
    scope = manifest.get("scope")
    by_name = {entry_name(item): item for item in entries if entry_name(item)}
    description = ""
    for name in SUMMARY_NAMES:
        entry = by_name.get(name)
        if not entry:
            continue
        content = verified_payload(package_dir, entry)
        if content is not None:
            description = first_summary_paragraph(content)
            break
    parts = [value for value in (scope if isinstance(scope, str) else "", description) if value]
    return scrub_summary(" | ".join(parts)) if parts else "No hash-verified summary payload declared."


def package_record(package_dir: Path) -> dict:
    manifest_path = next(
        (package_dir / name for name in MANIFEST_NAMES if (package_dir / name).is_file()),
        None,
    )
    if manifest_path is None:
        seal_path = package_dir / "package_seal.json"
        seal_hash = sha256_bytes(seal_path.read_bytes()) if seal_path.is_file() else None
        return {
            "package": package_dir.name,
            "manifest_sha256": seal_hash,
            "status": "NO_TOP_LEVEL_MANIFEST",
            "payloads": sorted(path.name for path in package_dir.iterdir() if path.is_file()),
            "summary": "Not cataloged as a verified implementer supplement.",
        }

    raw = manifest_path.read_bytes()
    try:
        manifest = json.loads(raw)
    except json.JSONDecodeError:
        return {
            "package": package_dir.name,
            "manifest_sha256": sha256_bytes(raw),
            "status": "MANIFEST_PARSE_ERROR",
            "payloads": [],
            "summary": "Manifest could not be parsed.",
        }

    entries = manifest_entries(manifest)
    names = [entry_name(item) for item in entries]
    return {
        "package": manifest.get("package_id") or manifest.get("package") or package_dir.name,
        "directory": package_dir.name,
        "manifest_sha256": sha256_bytes(raw),
        "manifest_schema": manifest.get("schema", "UNDECLARED"),
        "status": explicit_status(package_dir, manifest, entries),
        "payload_count": len(entries),
        "payloads": [name for name in names[:PAYLOAD_NAME_LIMIT] if name],
        "more_payloads": max(0, len(names) - PAYLOAD_NAME_LIMIT),
        "summary": package_summary(package_dir, manifest, entries),
    }


def markdown_cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\r", " ").replace("\n", " ")


def render_catalog(records: list[dict]) -> str:
    manifest_count = sum("MANIFEST_PARSE_ERROR" not in item["status"] and
                         item["status"] != "NO_TOP_LEVEL_MANIFEST" for item in records)
    lines = [
        "# Washed supplement catalog",
        "",
        "> Generated locator only. A catalog row is not evidence. Read the selected package, verify its manifest and payload hashes, then use only its stated scope.",
        "",
        f"Package directories: {len(records)}. Top-level manifests: {manifest_count}.",
        "",
        "Search by function, parameter, status, or package name:",
        "",
        "```powershell",
        'rtk rg -ni "F06OF|measurement receipt" outputs/IMPLEMENTER_SUPPLEMENTS/CATALOG.md',
        'rtk rg -ni "UNBOUND|NOT_RUN" outputs/IMPLEMENTER_SUPPLEMENTS/CATALOG.md',
        "```",
        "",
        "Rebuild with `rtk proxy python -B scripts/build_supplement_catalog.py`.",
        "",
        "| Package | Manifest SHA-256 | Declared status | Payloads | Scope summary |",
        "|---|---|---|---|---|",
    ]
    for item in records:
        payloads = ", ".join(item.get("payloads", []))
        if item.get("more_payloads"):
            payloads += f", +{item['more_payloads']} more"
        lines.append(
            "| " + " | ".join(
                markdown_cell(str(item.get(key, "")))
                for key in ("package", "manifest_sha256", "status")
            ) + " | " + markdown_cell(payloads) + " | " +
            markdown_cell(item.get("summary", "")) + " |"
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Build or verify the washed-supplement locator catalog.")
    parser.add_argument("--check", action="store_true", help="fail if the catalog is missing or stale")
    args = parser.parse_args()

    if not SUPPLEMENT_ROOT.is_dir():
        raise SystemExit(f"supplement directory not found: {SUPPLEMENT_ROOT}")
    package_dirs = sorted(path for path in SUPPLEMENT_ROOT.iterdir() if path.is_dir())
    records = [package_record(path) for path in package_dirs]
    expected_catalog = render_catalog(records)
    if args.check:
        actual_catalog = CATALOG_PATH.read_text(encoding="utf-8") if CATALOG_PATH.is_file() else None
        if actual_catalog != expected_catalog:
            print("supplement catalog is missing or stale", file=sys.stderr)
            return 1
    else:
        CATALOG_PATH.write_text(expected_catalog, encoding="utf-8", newline="\n")
    print(json.dumps({
        "catalog": str(CATALOG_PATH.relative_to(REPOSITORY_ROOT)),
        "package_directories": len(records),
        "top_level_manifests": sum(item["status"] != "NO_TOP_LEVEL_MANIFEST" for item in records),
        "index_is_locator_only": True,
        "catalog_is_current": True,
    }, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
