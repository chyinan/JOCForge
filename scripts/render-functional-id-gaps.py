# pattern: Imperative Shell
"""Render the current per-functional-ID implementation gap ledger."""

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IMPLEMENTATION_MAP = ROOT / "docs/implementation/IMPLEMENTATION-MAP.json"
OUTPUT = ROOT / "docs/implementation/FUNCTIONAL-ID-GAPS.md"
EXPECTED_PACKAGE_MANIFEST_SHA256 = (
    "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def package_entry(manifest: dict, relative_path: str) -> dict | None:
    return next(
        (entry for entry in manifest["files"] if entry["file"] == relative_path),
        None,
    )


def format_list(values: list[str]) -> str:
    return ", ".join(f"`{value}`" for value in values) if values else "无"


def format_aspects(module: dict) -> str:
    grouped: dict[str, list[str]] = {}
    for aspect in module.get("aspects", []):
        grouped.setdefault(aspect["status"], []).append(aspect["aspect"])
    if not grouped:
        return "矩阵未给出 aspect 级记录。"
    return "; ".join(
        f"`{status}`：{', '.join(names)}"
        for status, names in sorted(grouped.items())
    )


def render(package_root: Path) -> str:
    manifest_bytes = (package_root / "MANIFEST.json").read_bytes()
    manifest_sha256 = sha256(manifest_bytes)
    if manifest_sha256 != EXPECTED_PACKAGE_MANIFEST_SHA256:
        raise ValueError(
            "root package MANIFEST identity differs from the reviewed package"
        )
    manifest = json.loads(manifest_bytes)

    matrix_path = package_root / "SPEC-COMPLETENESS-MATRIX.json"
    matrix_bytes = matrix_path.read_bytes()
    matrix_entry = package_entry(manifest, "SPEC-COMPLETENESS-MATRIX.json")
    if matrix_entry is None:
        raise ValueError("SPEC-COMPLETENESS-MATRIX.json is absent from MANIFEST")
    if len(matrix_bytes) != matrix_entry["bytes"]:
        raise ValueError("SPEC-COMPLETENESS-MATRIX byte count does not match MANIFEST")
    matrix_sha256 = sha256(matrix_bytes)
    if matrix_sha256 != matrix_entry["sha256"]:
        raise ValueError("SPEC-COMPLETENESS-MATRIX SHA-256 does not match MANIFEST")
    matrix = json.loads(matrix_bytes)
    matrix_modules = {row["functional_id"]: row for row in matrix["modules"]}

    implementation_map = json.loads(IMPLEMENTATION_MAP.read_text(encoding="utf-8"))
    entries = implementation_map["functional_entries"]
    if len(entries) != 104 or len({entry["functional_id"] for entry in entries}) != 104:
        raise ValueError("local functional map must contain 104 unique IDs")

    output = [
        "# 未闭合 Functional ID 状态",
        "",
        "本报告逐项对照本地实现映射和批准包的完整性矩阵。父 Functional ID 只有明确完成时才算闭合；局部 direct slice 通过，不会替代父 ID 的完成状态。",
        "",
        f"- Functional ID 数量：{len(entries)}。",
        f"- root package MANIFEST SHA-256：`{manifest_sha256}`。",
        f"- SPEC-COMPLETENESS-MATRIX SHA-256：`{matrix_sha256}`。",
        f"- 本地映射文件：`{IMPLEMENTATION_MAP.relative_to(ROOT).as_posix()}`。",
        "- `测试`列只列出本地 direct slice 的定向测试；矩阵中的 unit-vector ID 是洗白包证据，不表示本地已执行该功能。",
        "",
    ]

    missing_matrix_ids = []
    for entry in entries:
        functional_id = entry["functional_id"]
        module = matrix_modules.get(functional_id)
        if module is None:
            missing_matrix_ids.append(functional_id)
            continue

        spec_identities = []
        for spec_file in entry.get("spec_files", []):
            spec_entry = package_entry(manifest, spec_file)
            if spec_entry is None:
                spec_identities.append(f"`{spec_file}`：MANIFEST 未登记")
            else:
                spec_identities.append(
                    f"`{spec_file}` SHA-256 `{spec_entry['sha256']}`"
                )

        direct_slices = entry.get("direct_slices", [])
        tests = sorted(
            {
                test
                for direct in direct_slices
                for test in direct.get("test_modules", [])
            }
            | set(entry.get("test_modules", []))
        )
        unit_vectors = module.get("delivered_unit_vectors", [])
        blocking_questions = module.get("blocking_questions", [])
        dependencies = module.get("dependencies", [])

        output.extend(
            [
                f"## {functional_id}",
                "",
                f"- 功能范围：`{module.get('function', '未登记')}`。",
                f"- 本地实现状态：`{entry['implementation_status']}`。",
                f"- 洗白规格状态：`{module['status']}`；问题：{format_list(blocking_questions)}；依赖：{format_list(dependencies)}。",
                f"- 规格身份：{'；'.join(spec_identities) if spec_identities else '无规格文件登记'}。",
                f"- 矩阵 aspect 状态：{format_aspects(module)}",
                f"- 洗白包 unit-vector ID：{format_list(unit_vectors)}。",
                f"- 洗白包覆盖说明：`{module.get('actual_rule_by_rule_coverage', '未登记')}`。",
                f"- 本地定向测试：{format_list(tests)}。",
            ]
        )

        if direct_slices:
            output.append("- 已实现的局部 slice 及父 ID 剩余范围：")
            for direct in direct_slices:
                output.append(
                    f"  - `{direct['slice_id']}`，状态 `{direct['implementation_status']}`：{direct['scope']}"
                )
                for case_pointer in direct.get("vectors", {}).get("case_pointers", []):
                    output.append(f"    - 向量说明：{case_pointer}")
        else:
            output.append(
                "- 局部实现：当前映射尚未登记可核验的 direct slice 或对应定向测试。"
            )
        output.append("")

    if missing_matrix_ids:
        raise ValueError(
            "Functional IDs absent from SPEC-COMPLETENESS-MATRIX: "
            + ", ".join(missing_matrix_ids)
        )
    return "\n".join(output)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--package-root",
        type=Path,
        required=True,
        help="path to the approved IMPLEMENTER_PACKAGE directory",
    )
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    rendered = render(args.package_root.resolve())
    if args.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != rendered:
            print("functional ID gap report differs from current sources")
            return 1
    else:
        OUTPUT.write_text(rendered, encoding="utf-8", newline="\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
