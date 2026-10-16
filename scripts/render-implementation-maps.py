# pattern: Imperative Shell
import argparse
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs" / "implementation"
MAP_PATH = DOCS / "IMPLEMENTATION-MAP.json"
QUOTE = chr(96)


def inline(value):
    return f"{QUOTE}{value}{QUOTE}"


def refs(values):
    if not values:
        return "—"
    return "<br>".join(inline(value) for value in values)


def table(title, columns, rows):
    header = f"# {title}\n\nGenerated from IMPLEMENTATION-MAP.json. Do not edit by hand.\n\n"
    head = "| " + " | ".join(columns) + " |\n"
    rule = "|" + "|".join("---" for _ in columns) + "|\n"
    body = "".join(
        "| " + " | ".join(str(cell).replace("|", "\\|") for cell in row) + " |\n"
        for row in rows
    )
    return header + head + rule + body


def render(data):
    entries = data["functional_entries"]
    output = {}

    output["SPEC-TRACEABILITY.md"] = table(
        "Functional specification traceability",
        [
            "functional_id",
            "spec_files",
            "owner",
            "planned_phase",
            "status",
            "scoped_clauses",
            "implementation_files",
            "test_modules",
        ],
        [
            [
                entry["functional_id"],
                refs(entry["spec_files"]),
                inline(entry["owner"]),
                entry["planned_phase"],
                entry["status"],
                refs(entry.get("clock", {}).get("covered_clauses", [])),
                refs(entry.get("implementation_files", [])),
                refs(entry.get("test_modules", [])),
            ]
            for entry in entries
        ],
    )

    output["OWNER-MAP.md"] = table(
        "Functional owner map",
        ["functional_id", "owner", "status"],
        [
            [entry["functional_id"], inline(entry["owner"]), entry["status"]]
            for entry in entries
        ],
    )

    output["CLOCK-MAP.md"] = table(
        "Clock scope map",
        ["functional_id", "domains", "covered_clauses", "status"],
        [
            [
                entry["functional_id"],
                refs(entry.get("clock", {}).get("domains", [])),
                refs(entry.get("clock", {}).get("covered_clauses", [])),
                entry["status"],
            ]
            for entry in entries
        ],
    )

    error_rows = []
    for entry in entries:
        outcomes = entry.get("error_outcomes", [])
        if outcomes:
            for outcome in outcomes:
                error_rows.append(
                    [
                        entry["functional_id"],
                        entry["status"],
                        inline(outcome["error_code"]),
                        outcome["code_authority"],
                        outcome["trigger"],
                        outcome["category"],
                        outcome["state_effect"],
                    ]
                )
        elif entry["status"] == "DEFERRED_DETAILED_MAPPING":
            error_rows.append(
                [
                    entry["functional_id"],
                    "DEFERRED_DETAILED_MAPPING",
                    "—",
                    "—",
                    "—",
                    "—",
                    "—",
                ]
            )
        else:
            error_rows.append(
                [
                    entry["functional_id"],
                    entry["status"],
                    "—",
                    "—",
                    "—",
                    "—",
                    "No additional outcome is mapped for this scoped clause.",
                ]
            )
    output["ERROR-MAP.md"] = table(
        "Error outcome map",
        [
            "functional_id",
            "mapping_status",
            "local_error_label",
            "label_authority",
            "trigger",
            "category",
            "state_effect",
        ],
        error_rows,
    )

    output["PARAMETER-MAP.md"] = table(
        "Parameter scope map",
        ["functional_id", "mapping_status", "kind", "provider_files"],
        [
            [
                entry["functional_id"],
                entry["status"],
                entry.get("parameters", {}).get("kind", "—"),
                refs(entry.get("parameters", {}).get("provider_files", [])),
            ]
            for entry in entries
        ],
    )

    output["VECTOR-MAP.md"] = table(
        "Vector scope map",
        [
            "functional_id",
            "mapping_status",
            "kind",
            "parent_vectors",
            "case_pointers",
            "direct_package_vector",
        ],
        [
            [
                entry["functional_id"],
                entry["status"],
                entry.get("vectors", {}).get("kind", "—"),
                refs(entry.get("vectors", {}).get("parent_vectors", [])),
                refs(entry.get("vectors", {}).get("case_pointers", [])),
                entry.get("vectors", {}).get("direct_package_vector", "—"),
            ]
            for entry in entries
        ],
    )

    phase = data["phase_one"]["P1.1"]
    task_rows = []
    for task in phase["tasks"]:
        task_rows.append(
            [
                task["task_id"],
                json.dumps(task.get("functional_scope", {}), ensure_ascii=False),
                refs(task.get("covered_clauses", [])),
                refs(task.get("planned_files", [])),
                refs(task.get("planned_tests", [])),
                task["status"],
            ]
        )
    plan = table(
        "Phase 1 plan",
        [
            "task_id",
            "functional_scope",
            "covered_clauses",
            "planned_files",
            "planned_tests",
            "status",
        ],
        task_rows,
    )
    plan += "\n## Gate order\n\n"
    plan += "The documentation gate must pass before CLOCK-FOUNDATION implementation work begins. "
    plan += "CLOCK-FOUNDATION is the only GO task in this slice.\n\n"
    plan += "P1.2, P1.3, P1.4, and SELECTED-PROFILE-GATE remain HOLD.\n\n"
    plan += "## Failing-first tests\n\n"
    plan += "\n".join(
        f"{index}. {test}"
        for index, test in enumerate(phase["failing_first_tests"], start=1)
    )
    plan += "\n"
    output["PHASE-PLAN.md"] = plan
    return output


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    data = json.loads(MAP_PATH.read_text(encoding="utf-8"))
    mismatches = []
    for name, contents in render(data).items():
        path = DOCS / name
        if args.check:
            if not path.is_file() or path.read_text(encoding="utf-8") != contents:
                mismatches.append(name)
        else:
            path.write_text(contents, encoding="utf-8", newline="\n")
    if mismatches:
        print("generated maps differ: " + ", ".join(mismatches), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
