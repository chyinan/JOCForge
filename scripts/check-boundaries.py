# pattern: Imperative Shell
"""Validate approved architecture, mappings, dependencies, and empty boundaries."""
from __future__ import annotations

import json
import pathlib
import re
import sys
import tomllib


class BoundaryError(RuntimeError):
    """A repository boundary is missing or inconsistent."""


ROOT = pathlib.Path(__file__).resolve().parents[1]
PACKAGE = pathlib.Path(
    r"C:\Users\chyinan\Documents\Codex\2026-10-12\files-pasted-by-the-user-agent"
    r"\outputs\IMPLEMENTER_PACKAGE"
)


def check(condition: bool, message: str) -> None:
    if not condition:
        raise BoundaryError(message)


def read_text(path: pathlib.Path) -> str:
    check(path.is_file(), f"missing file: {path}")
    return path.read_text(encoding="utf-8")


def parse_markdown_table(path: pathlib.Path, expected_columns: list[str]) -> list[dict[str, str]]:
    lines = read_text(path).splitlines()
    header_index = next(
        (index for index, line in enumerate(lines) if line.startswith("| ") and all(column in line for column in expected_columns)),
        None,
    )
    check(header_index is not None, f"missing table header in {path.name}")
    header = [cell.strip() for cell in lines[header_index].strip("|").split("|")]
    check(header == expected_columns, f"wrong columns in {path.name}: {header}")
    rows: list[dict[str, str]] = []
    for line in lines[header_index + 2 :]:
        if not line.startswith("|"):
            break
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        check(len(cells) == len(header), f"wrong cell count in {path.name}: {line}")
        rows.append(dict(zip(header, cells, strict=True)))
    return rows


def split_refs(cell: str) -> list[str]:
    if cell == "—":
        return []
    return [part.strip().strip("`") for part in cell.split("<br>")]


def check_unique_ids(rows: list[dict[str, str]], expected_ids: set[str], name: str) -> None:
    ids = [row["functional_id"] for row in rows]
    check(len(ids) == 104, f"{name} has {len(ids)} functional rows, expected 104")
    check(len(set(ids)) == len(ids), f"{name} has duplicate functional IDs")
    check(set(ids) == expected_ids, f"{name} functional ID set differs from authority")


def check_source_api(source: pathlib.Path, text: str) -> None:
    forbidden = re.search(r"include!|#\[path|static\s+mut|legacy|ffmpeg|compat|fallback", text, re.I)
    check(forbidden is None, f"forbidden production reference in {source}: {forbidden.group(0) if forbidden else ''}")
    declarations = re.findall(
        r"\bpub\s+(?:async\s+)?(?:fn|struct|enum|type)\b(?:(?![{};]).|\n){0,500}[{;]",
        text,
    )
    for declaration in declarations:
        check(re.search(r"\busize\b", declaration) is None, f"public usize in {source}: {declaration!r}")


def run_checks(root: pathlib.Path = ROOT, package: pathlib.Path = PACKAGE) -> None:
    architecture = root / "docs" / "architecture"
    implementation = root / "docs" / "implementation"
    required_architecture = {
        "APPROVAL.md", "ARCHITECTURE.md", "ARCHITECTURE-DECISIONS.md", "DATAFLOW.md",
        "DEPENDENCY-RULES.md", "ERROR-MODEL.md", "MODULE-MAP.md", "OLD-WIRE-REUSE-MATRIX.md",
        "PROFILE-DIMENSIONS.md", "PUBLIC-API.md", "STATE-OWNERSHIP.md", "TEST-STRATEGY.md",
    }
    required_maps = {
        "CLOCK-MAP.md", "ERROR-MAP.md", "OWNER-MAP.md", "PARAMETER-MAP.md",
        "PHASE-PLAN.md", "SPEC-TRACEABILITY.md", "VECTOR-MAP.md",
    }
    check(required_architecture <= {path.name for path in architecture.glob("*.md")}, "missing approved architecture document")
    check(required_maps <= {path.name for path in implementation.glob("*.md")}, "missing implementation map")

    expected_dependencies = {
        "domain": set(), "application": {"jocforge-domain"}, "io": {"jocforge-domain"},
        "cli": {"jocforge-application"},
        "validation": {"jocforge-domain", "jocforge-application", "jocforge-io"},
    }
    workspace = tomllib.loads(read_text(root / "Cargo.toml"))
    check(set(workspace["workspace"]["members"]) == {f"crates/jocforge-{name}" for name in expected_dependencies}, "workspace crate set differs")
    for name, expected in expected_dependencies.items():
        crate = root / "crates" / f"jocforge-{name}"
        config = tomllib.loads(read_text(crate / "Cargo.toml"))
        dependencies = config.get("dependencies", {})
        check(set(dependencies) == expected, f"reverse or unregistered dependency in {name}")
        check(name == "validation" or "jocforge-validation" not in dependencies, f"production depends on validation: {name}")
        check(not config.get("features"), f"algorithm feature variants in {name}")
        check(not config.get("build-dependencies"), f"build dependency in {name}")
        for source in (crate / "src").rglob("*.rs"):
            text = read_text(source)
            check_source_api(source, text)
            if name in {"domain", "application"}:
                io_ref = re.search(r"\b(?:std::)?(?:fs|process|env|net)::|Command\s*::", text)
                check(io_ref is None, f"domain/application I/O in {source}")

    profile = read_text(architecture / "PROFILE-DIMENSIONS.md")
    for dimension in ("AuthoredInputProfile", "CoreCarrierProfile", "JocDownmixProfile", "SubstreamTopology", "CodecRateProfile", "EmdfCarriageProfile", "RenderTarget"):
        check(dimension in profile, f"missing profile dimension: {dimension}")
    supported = [line for line in profile.splitlines() if line.startswith("| selected ADM BWF /")]
    check(len(supported) == 1, "supported profile row must be unique")
    modes = set(re.findall(r"`([^`]+)`", supported[0].split("|")[6]))
    check(modes == {"deployed_compat", "etsi_emdf"}, "unregistered or missing carriage variant")
    check("UNSUPPORTED_PROFILE" in profile, "profile rejection is undocumented")

    matrix = json.loads(read_text(package / "SPEC-COMPLETENESS-MATRIX.json"))
    expected_ids = {row["functional_id"] for row in matrix["modules"] if row["readiness_stage"] == "PRE_HANDOFF_SPEC"}
    check(len(expected_ids) == 104, "authority does not expose 104 PRE_HANDOFF IDs")

    trace_columns = ["functional_id", "spec_file", "implementation_files", "test_files", "owner", "clock_domain", "parameter_kind", "parameter_sources", "vector_kind", "vector_sources", "review_evidence", "parent_integration_coverage", "planned_phase"]
    trace = parse_markdown_table(implementation / "SPEC-TRACEABILITY.md", trace_columns)
    check_unique_ids(trace, expected_ids, "SPEC-TRACEABILITY")
    trace_by_id = {row["functional_id"]: row for row in trace}
    for row in trace:
        for ref in split_refs(row["spec_file"]):
            check((package / ref).is_file(), f"invalid spec path for {row['functional_id']}: {ref}")
        check(row["implementation_files"] != "—", f"missing planned implementation files: {row['functional_id']}")
        check(row["test_files"] != "—", f"missing planned behavioral tests: {row['functional_id']}")

    parameter_columns = ["functional_id", "provider_kind", "provider_files", "generation_specs", "registry_entries", "validation_rules", "runtime_loader", "status"]
    parameters = parse_markdown_table(implementation / "PARAMETER-MAP.md", parameter_columns)
    check_unique_ids(parameters, expected_ids, "PARAMETER-MAP")
    provider_kinds = {"FILE_PROVIDER", "GENERATED_PROVIDER", "COMPOSITE_PROVIDER", "NO_RUNTIME_PARAMETER", "BLOCKED_MISSING_PROVIDER"}
    for row in parameters:
        check(row["provider_kind"] in provider_kinds, f"invalid provider kind: {row}")
        providers = split_refs(row["provider_files"])
        check(all("PARAMETER-INTERFACES.json" not in ref for ref in providers), f"registry used as provider: {row['functional_id']}")
        for ref in providers + split_refs(row["generation_specs"]):
            check((package / ref).is_file(), f"invalid provider/spec path for {row['functional_id']}: {ref}")
        for ref in split_refs(row["registry_entries"]):
            check(ref.startswith("PARAMETER-INTERFACES.json#"), f"invalid registry entry: {ref}")
        check(row["provider_kind"] != "NO_RUNTIME_PARAMETER" or not providers, f"NO_RUNTIME_PARAMETER has provider: {row['functional_id']}")

    vector_columns = ["functional_id", "vector_kind", "authoritative_vectors", "parent_integration_vectors", "review_evidence", "coverage_ledger", "planned_test_module"]
    vectors = parse_markdown_table(implementation / "VECTOR-MAP.md", vector_columns)
    check_unique_ids(vectors, expected_ids, "VECTOR-MAP")
    vector_kinds = {"DIRECT_VECTOR", "PARENT_INTEGRATION_VECTOR", "RULE_AND_PROPERTY_TEST", "NO_DIRECT_VECTOR", "BLOCKED_MISSING_VECTOR"}
    for row in vectors:
        check(row["vector_kind"] in vector_kinds, f"invalid vector kind: {row}")
        authoritative = split_refs(row["authoritative_vectors"])
        check(all("PRE-HANDOFF-EVIDENCE-COVERAGE.json" not in ref for ref in authoritative), f"coverage ledger used as direct vector: {row['functional_id']}")
        check(row["vector_kind"] != "DIRECT_VECTOR" or authoritative, f"DIRECT_VECTOR has no vector: {row['functional_id']}")
        for ref in authoritative + split_refs(row["parent_integration_vectors"]) + split_refs(row["review_evidence"]) + split_refs(row["coverage_ledger"]):
            check((package / ref).is_file(), f"invalid vector/evidence path for {row['functional_id']}: {ref}")
        check(row["planned_test_module"] != "—", f"missing planned test module: {row['functional_id']}")

    error_columns = ["functional_id", "error_code", "trigger", "category", "staged_state_created", "committed_state_changed", "candidate_search_continues", "programme_state_after", "reset_required", "caller_action", "planned_error_variant", "spec_reference"]
    errors = parse_markdown_table(implementation / "ERROR-MAP.md", error_columns)
    check_unique_ids(errors, expected_ids, "ERROR-MAP")
    categories = {"INPUT_REJECTION", "UNSUPPORTED_PROFILE", "PARAMETER_UNAVAILABLE", "PARAMETER_IDENTITY", "CANDIDATE_INFEASIBLE", "NO_ACCEPTABLE_CANDIDATE", "TRANSACTION_ABORT", "FATAL_FAIL_STOP", "DURABLE_IO_FAILURE", "RETIRED_GUARD"}
    check({row["category"] for row in errors} == categories, "ERROR-MAP does not distinguish every required category")
    for row in errors:
        check(row["category"] in categories, f"invalid error category: {row}")
        check(row["committed_state_changed"] == "NO", f"failure changes committed state: {row['functional_id']}")
        check((package / row["spec_reference"].strip("`")).is_file(), f"invalid error spec: {row['functional_id']}")

    owner_columns = ["functional_id", "owner", "receipt_scope", "implementation_files"]
    owners = parse_markdown_table(implementation / "OWNER-MAP.md", owner_columns)
    check_unique_ids(owners, expected_ids, "OWNER-MAP")
    for row in owners:
        check(row["owner"] == trace_by_id[row["functional_id"]]["owner"], f"owner mismatch: {row['functional_id']}")

    clock_columns = ["functional_id", "clock_domains", "public_index_types", "conversion_evidence"]
    clocks = parse_markdown_table(implementation / "CLOCK-MAP.md", clock_columns)
    check_unique_ids(clocks, expected_ids, "CLOCK-MAP")
    for row in clocks:
        check(row["clock_domains"] == trace_by_id[row["functional_id"]]["clock_domain"], f"clock mismatch: {row['functional_id']}")
        check(row["public_index_types"] != "—", f"missing typed clock: {row['functional_id']}")

    phase_columns = ["Slice", "Functional IDs", "Spec/vector", "Planned files", "Planned behavioral tests", "Expected first failure", "Minimal implementation", "Targeted verification", "Workspace verification", "Commit message", "Independent review gate", "Exit criteria"]
    phase_rows = parse_markdown_table(implementation / "PHASE-PLAN.md", phase_columns)
    check([row["Slice"].split()[0] for row in phase_rows] == ["P1.1", "P1.2", "P1.3", "P1.4"], "Phase 1 must contain exactly four ordered slices")
    exclusions = read_text(implementation / "PHASE-PLAN.md")
    for excluded in ("FFT", "QMF", "BAP", "mantissa", "Huffman", "EMDF/OAMD/JOC payload", "frame writer", "PCM/ADM reader", "DSP", "future profiles"):
        check(excluded in exclusions, f"missing Phase 1 exclusion: {excluded}")

    projection = json.loads(read_text(package / "parameters" / "selected-owner-projection-v1.json"))
    check(len(projection["field_groups"]) == 7, "selected overlap projection must have 7 groups")
    check(len({group["receipt_owner"] for group in projection["field_groups"]}) == 7, "receipt owner labels must be unique")
    check(len({group["commit_owner"] for group in projection["field_groups"]}) == 6, "commit-owner labels must have cardinality 6")
    fields: set[str] = set()
    for group in projection["field_groups"]:
        for field in group["fields"]:
            check(field not in fields, f"committed field has multiple owners: {field}")
            fields.add(field)

    state_text = read_text(architecture / "STATE-OWNERSHIP.md")
    root_schema = json.loads(read_text(package / "test-vectors" / "GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json"))
    expected_roots = set(root_schema["properties"]["owners"]["required"])
    documented_roots = set(re.findall(r"^\| `([a-z_]+)` \|", state_text, re.MULTILINE))
    check(expected_roots <= documented_roots, f"missing programme owner roots: {expected_roots - documented_roots}")
    check("none_pure_value` 不属于 committed owner" in state_text, "pure proposal ownership is unclear")
    check("不是额外 leaf owner" in state_text, "programme root is misrepresented as a leaf owner")

    empty_leafs = list((root / "crates" / "jocforge-domain" / "src" / "codec").glob("*.rs"))
    empty_leafs += list((root / "crates" / "jocforge-domain" / "src" / "integrity").glob("*.rs"))
    empty_leafs += [path for path in (root / "crates" / "jocforge-domain" / "src" / "joc").glob("*.rs") if path.name != "mod.rs"]
    for path in empty_leafs:
        text = read_text(path)
        check(text.startswith("// pattern: Functional Core"), f"missing FCIS classification: {path}")
        declaration = re.search(r"^\s*(?:pub\s+)?(?:async\s+)?(?:fn|struct|enum|const|static|type)\b", text, re.MULTILINE)
        check(declaration is None, f"empty leaf contains implementation: {path}")


def main() -> int:
    try:
        run_checks()
    except BoundaryError as error:
        print(f"boundary check failed: {error}", file=sys.stderr)
        return 1
    print("architecture and source boundary checks passed (104 IDs; corrected parameter/vector/error maps; 12 programme roots; explicit gates)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
