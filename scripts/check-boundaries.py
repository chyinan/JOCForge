# pattern: Imperative Shell
import importlib.util
import json
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
PACKAGE_ROOT = pathlib.Path(
    r"C:\Users\chyinan\Documents\Codex\2026-10-12\files-pasted-by-the-user-agent\outputs\IMPLEMENTER_PACKAGE"
)

EXPECTED_ROOT_IDS = [
    "reader_eof",
    "frontend_control",
    "spatial_control",
    "spatial_audio",
    "joc",
    "meter",
    "drc",
    "analysis",
    "codec",
    "metadata",
    "integrity",
    "output",
]
EXPECTED_FUNCTIONAL_IDS = set(
    "STRC,GTC,XSTJ,F01,F01R,F01G,F01B,F01GL,F01C,F01E,F02,F03,F03G,F03C,F03U,F03F,F03FC,F03R,F03P,F03N,F03S,F03FD,F03W,F04,F04C,F04A,F04F,F04FM,F04FCM,F04G,F04GA,F04P,F04PG,F04R,F04D,F04Q,F04RB,F04QF,F04QP,F05,F05Q,F05QH,F05J,F05JC,F05JL,F05JS,F05O,F05T,F05H,F05L,F05R56,F06,F06L,F06I,F06W,F06KG,F06TP,F06ND,F06NR,F06NI,F06NF,F06WH,F06U,F06DG,F06D,F06M,F06OG,F06OM,F06OP,F06OE,F06QT,F06QM,F06QS,F06OD,F06OS,F06OB,F06OR,F06OC,F06OF,F07,F07A,F08,F08NZ,F09,F09E,F09PA,F09G,F09LB,F10,F11,F12,F12S,F12SC,F12A,F13,F13F,F13T,F13C,F13D2,F14,F14K,F15,F15P,OWNJ".split(",")
)
CLOCK_FILES = [
    "crates/jocforge-domain/src/timeline/index.rs",
    "crates/jocforge-domain/src/timeline/clock.rs",
    "crates/jocforge-domain/src/timeline/mod.rs",
]
CLOCK_TESTS = ["crates/jocforge-domain/tests/timeline_clock_contract.rs"]
CLOCK_DOMAINS = [
    "AdmSampleCount",
    "NormalCallbackCount",
    "FrontendCallbackIndex",
    "CodecSourceIndex",
    "JocDispatchIndex",
    "JocGeneratedWriterIndex",
    "OrdinaryPayloadWriterIndex",
    "JocPoppedWriterIndex",
    "PublishedAuIndex",
]
CLOCK_CLAUSES = [
    "F02.1: each callback presents a complete 1536-sample host frame; EOF padding is not a codec flush",
    "F02.2: N>0; A=ceil(N/1536); P=1536*A-N",
    "F02.2: normal callbacks g=0..A-1; four cached drains g=A..A+3",
    "F02.2/F02.3: source s=g-2 exists only when g>=2",
    "XSTJ CLOCK: dispatch d=s+1=g-1; generated JOC writer d-1=s",
    "XSTJ J03: source 0 and 1 complete queue/source processing but suppress AU publication",
    "XSTJ J03: source 1 pops JOC writer 0 as CONSUMED_SUPPRESSED",
    "XSTJ J03: source s>=2 publishes AU j=s-2",
    "XSTJ CLOCK: publication consumes ordinary writer j and popped JOC writer s-1",
    "F02.2: A+4 callbacks produce A+2 source frames and A published AUs",
    "F02.3: EOF adds four cached drains and no codec flush",
    "F02.4: N=6522906 yields 4247 normal callbacks, 4 drains, 4249 sources, 4247 AUs",
    "F02.4: for N=6522906, normal callback g=0..4246 and cached drain g=4247..4250",
    "XSTJ CLOCK: for N=6522906, source s=0..4248; dispatch d=1..4249; generated JOC writer s=0..4248",
    "XSTJ CLOCK: for N=6522906, published AU j=0..4246; ordinary writer j=0..4246; popped JOC writer s-1=1..4247",
    "terminal pending writers remain ordinary 4247/4248 and JOC writer 4248",
]
ACTIVE_ENTRY_FIELDS = {
    "functional_id",
    "spec_files",
    "implementation_files",
    "test_modules",
    "owner",
    "clock",
    "parameters",
    "vectors",
    "error_outcomes",
    "planned_phase",
    "phase_scope",
    "status",
    "inventory_status",
    "implementation_status",
    "active_slice_materialization",
}
DEFERRED_ENTRY_FIELDS = {
    "functional_id",
    "spec_files",
    "owner",
    "planned_phase",
    "status",
}


class BoundaryError(RuntimeError):
    pass


def require(condition, message):
    if not condition:
        raise BoundaryError(message)


def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise BoundaryError(f"cannot read JSON {path}: {error}") from error


def check_active_mapping(root, package, data):
    entries = data.get("functional_entries")
    require(isinstance(entries, list) and len(entries) == 104, "functional ID count must be exactly 104")
    identifiers = [entry.get("functional_id") for entry in entries]
    require(len(set(identifiers)) == 104, "functional IDs must be unique")
    require(set(identifiers) == EXPECTED_FUNCTIONAL_IDS, "functional ID inventory differs from the approved 104 IDs")
    require(all(isinstance(identifier, str) and identifier for identifier in identifiers), "functional ID is empty")

    active = {"F02", "XSTJ"}
    require(
        {entry["functional_id"] for entry in entries if entry.get("active_slice_materialization") == "MATERIALIZED"}
        == active,
        "only the scoped F02 and XSTJ clock entries may be materialized",
    )
    for entry in entries:
        identifier = entry["functional_id"]
        for spec_file in entry.get("spec_files", []):
            require((package / spec_file).is_file(), f"{identifier} spec file is missing: {spec_file}")
        if identifier not in active:
            require(set(entry) == DEFERRED_ENTRY_FIELDS, f"{identifier} future mapping contains unreviewed detail")
            require(entry["status"] == "DEFERRED_DETAILED_MAPPING", f"{identifier} is not deferred")
            require(bool(entry["spec_files"]), f"{identifier} has no spec file")
            continue

        require(set(entry) == ACTIVE_ENTRY_FIELDS, f"{identifier} active mapping field set drift")
        require(entry["status"] == "PARTIALLY_MATERIALIZED", f"{identifier} claims a complete mapping")
        require(entry["inventory_status"] == "INVENTORIED", f"{identifier} inventory status drift")
        require(
            entry["implementation_status"] == "IMPLEMENTATION_PENDING_A_GATE",
            f"{identifier} implementation status must wait for the A gate",
        )
        require(entry["clock"]["domains"] == CLOCK_DOMAINS, f"{identifier} clock-domain types drift")
        require(entry["clock"]["covered_clauses"] == CLOCK_CLAUSES, f"{identifier} clock clauses drift")
        require(entry["implementation_files"] == CLOCK_FILES, f"{identifier} planned file set drift")
        require(entry["test_modules"] == CLOCK_TESTS, f"{identifier} planned test file set drift")
        require(entry["phase_scope"] == ["CLOCK-FOUNDATION:SCOPED_CLAUSES_ONLY"], f"{identifier} scope drift")
        require(
            entry["parameters"]["kind"] == "NO_RUNTIME_PARAMETER"
            and entry["parameters"]["provider_files"] == []
            and entry["parameters"]["runtime_loader"] is None,
            f"{identifier} must not claim a provider or runtime parameter",
        )
        require(
            entry["vectors"]["kind"] == "RULE_AND_PROPERTY_TEST"
            and entry["vectors"]["authoritative_vectors"] == []
            and entry["vectors"]["direct_package_vector"] is None,
            f"{identifier} must not claim a direct package vector",
        )
        require(all((root / path).is_file() for path in CLOCK_FILES + CLOCK_TESTS), f"{identifier} planned files are missing")

    f02 = next(entry for entry in entries if entry["functional_id"] == "F02")
    xstj = next(entry for entry in entries if entry["functional_id"] == "XSTJ")
    require(
        f02["spec_files"] == ["specs/01-input-and-clock.md"]
        and xstj["spec_files"] == ["specs/00-cross-stage-contract-joins.md"],
        "active clock spec binding drift",
    )
    require(
        [item["error_code"] for item in f02["error_outcomes"]]
        == ["ZERO_SAMPLE_COUNT", "CLOCK_CONVERSION_RANGE"],
        "CLOCK error outcomes must cover only zero count and checked conversion range",
    )
    require(
        all(
            item["code_authority"] == "LOCAL_TRANSLATION_NOT_PACKAGE_NORMATIVE"
            for item in f02["error_outcomes"]
        ),
        "local CLOCK labels must not be claimed as package-normative codes",
    )
    require(xstj["error_outcomes"] == [], "XSTJ must not duplicate F02 input error outcomes")
    require(
        xstj["vectors"]["parent_vectors"] == ["test-vectors/CROSS-STAGE-CONTRACT-JOIN-VECTORS.json"]
        and xstj["vectors"]["case_pointers"]
        == [
            "J03_source0_absent_payload",
            "J03_source1_disposes_joc0",
            "J03_source2_first_published",
        ],
        "XSTJ J03 integration case mapping drift",
    )
    vector_file = read_json(package / "test-vectors/CROSS-STAGE-CONTRACT-JOIN-VECTORS.json")
    vector_cases = {case["case_id"]: case for case in vector_file["cases"]}
    source_zero = vector_cases["J03_source0_absent_payload"]
    source_one = vector_cases["J03_source1_disposes_joc0"]
    source_two = vector_cases["J03_source2_first_published"]
    require(
        source_zero["input"]["source"] == 0
        and source_zero["expected"]["source_committed"] is True
        and source_zero["expected"]["payload"] == "ABSENT_PUBLISHED_PAYLOAD_V1"
        and source_zero["expected"]["au_appended"] is False,
        "XSTJ J03 source-zero expected fields drift",
    )
    require(
        source_one["input"]["source"] == 1
        and source_one["expected"]["joc_writer0_disposition"] == "CONSUMED_SUPPRESSED"
        and source_one["expected"]["au_appended"] is False,
        "XSTJ J03 source-one expected fields drift",
    )
    require(
        source_two["input"]["source"] == 2 and source_two["expected"]["au_index"] == 0,
        "XSTJ J03 source-two expected fields drift",
    )

    phase = data["phase_one"]
    require(
        data["status"]["TRACEABILITY"] == "PARTIALLY_MATERIALIZED",
        "traceability must remain partial",
    )
    require(
        data["status"]["CLOCK_FOUNDATION"] == "GO_AFTER_A_GATE"
        and data["status"]["SELECTED_PROFILE_GATE"] == "HOLD"
        and phase["P1.1"]["status"] == "A_GATE_REQUIRED_BEFORE_CLOCK_IMPLEMENTATION",
        "P1.1 active-slice gate status drift",
    )
    require(
        [task["task_id"] for task in phase["P1.1"]["tasks"]]
        == ["CLOCK-FOUNDATION", "SELECTED-PROFILE-GATE"]
        and phase["P1.1"]["tasks"][0]["status"] == "GO"
        and phase["P1.1"]["tasks"][1] == {"task_id": "SELECTED-PROFILE-GATE", "status": "HOLD"},
        "P1.1 must have CLOCK-FOUNDATION as its only GO task",
    )
    require(
        phase["P1.1"]["tasks"][0]["functional_scope"]
        == {"F02": "SCOPED_CLAUSES_ONLY", "XSTJ": "SCOPED_CLAUSES_ONLY"}
        and phase["P1.1"]["tasks"][0]["covered_clauses"] == CLOCK_CLAUSES
        and phase["P1.1"]["tasks"][0]["planned_files"] == CLOCK_FILES
        and phase["P1.1"]["tasks"][0]["planned_tests"] == CLOCK_TESTS,
        "CLOCK-FOUNDATION plan drift",
    )
    require(phase["P1.1"]["completed_functional_ids"] == [], "CLOCK slice cannot mark F02 or XSTJ complete")
    require(
        all(data["status"][phase_id] == "HOLD" and phase[phase_id] == "HOLD" for phase_id in ("P1.2", "P1.3", "P1.4")),
        "P1.2, P1.3, and P1.4 must remain HOLD",
    )
    require(
        not {"P1.3-D", "P1.4-A", "P1.4-B"}.intersection(phase),
        "unreviewed future root/provider details must remain deferred",
    )

    roots = data.get("programme_roots", [])
    require(len(roots) == 12, "programme root inventory must contain exactly 12 roots")
    require([item.get("root_id") for item in roots] == EXPECTED_ROOT_IDS, "programme root inventory order or identity drift")
    for root_entry in roots:
        require(
            root_entry
            == {
                "root_id": root_entry["root_id"],
                "materialization_status": "INVENTORIED_PENDING_P1.3_MATERIALIZATION",
            },
            f"{root_entry.get('root_id')} root inventory contains unmaterialized detail",
        )
    require(
        data.get("selected_overlap_projection")
        == {"status": "INVENTORIED_PENDING_P1.3_MATERIALIZATION"},
        "overlap projection must remain deferred until P1.3 materialization",
    )

    test_source = (root / CLOCK_TESTS[0]).read_text(encoding="utf-8")
    required_test_markers = [
        "fn sample_count_accounting_matches_scoped_contract()",
        "fn callback_source_dispatch_and_generated_writer_are_checked()",
        "fn publication_distinguishes_generated_popped_and_ordinary_writers()",
        "fn terminal_pending_writers_are_explicit()",
        "6_522_906",
        "OrdinaryPayloadWriterIndex",
    ]
    require(all(marker in test_source for marker in required_test_markers), "CLOCK-FOUNDATION contract tests are incomplete")
    index_source = (root / CLOCK_FILES[0]).read_text(encoding="utf-8")
    require(
        all(
            f"index_type!({type_name});" in index_source
            for type_name in ("JocGeneratedWriterIndex", "OrdinaryPayloadWriterIndex", "JocPoppedWriterIndex")
        ),
        "CLOCK writer domains must use distinct newtypes",
    )
    return {"functional_ids": len(entries), "active_ids": len(active), "deferred_roots": len(roots)}


def check_project_boundaries(root):
    required_docs = [
        "docs/architecture/ARCHITECTURE.md",
        "docs/architecture/MODULE-MAP.md",
        "docs/architecture/STATE-OWNERSHIP.md",
        "docs/architecture/DATAFLOW.md",
        "docs/architecture/DEPENDENCY-RULES.md",
        "docs/architecture/ERROR-MODEL.md",
        "docs/architecture/TEST-STRATEGY.md",
        "docs/architecture/PROFILE-DIMENSIONS.md",
        "docs/architecture/PUBLIC-API.md",
        "docs/architecture/ARCHITECTURE-DECISIONS.md",
        "docs/architecture/APPROVAL.md",
        "docs/implementation/SPEC-TRACEABILITY.md",
        "docs/implementation/OWNER-MAP.md",
        "docs/implementation/CLOCK-MAP.md",
        "docs/implementation/ERROR-MAP.md",
        "docs/implementation/PARAMETER-MAP.md",
        "docs/implementation/VECTOR-MAP.md",
        "docs/implementation/PHASE-PLAN.md",
    ]
    for relative in required_docs:
        require((root / relative).is_file(), f"required architecture or mapping document is missing: {relative}")

    profile_doc = (root / "docs/architecture/PROFILE-DIMENSIONS.md").read_text(encoding="utf-8")
    supported_rows = [
        line for line in profile_doc.splitlines() if line.startswith("| selected ADM BWF /")
    ]
    require(len(supported_rows) == 1, "profile dimensions must register exactly one supported row")
    require("UNSUPPORTED_PROFILE" in profile_doc, "unsupported profile combinations must be rejected explicitly")
    production_profile = (root / "crates/jocforge-domain/src/profile/production.rs").read_text(encoding="utf-8")
    require("RenderTarget" not in production_profile, "RenderTarget must remain validation-only")
    require("fallback" not in production_profile.lower(), "production profile must not add fallback behavior")
    require(
        "Self::selected_with_carriage(EmdfCarriageProfile::DeployedCompat)" in production_profile,
        "deployed_compat must remain the production default",
    )

    dependency_sets = {
        "domain": {"sha2", "serde_json"},
        "application": {"jocforge-domain"},
        "io": {"jocforge-domain"},
        "cli": {"jocforge-application"},
        "validation": {"jocforge-domain", "jocforge-application", "jocforge-io"},
    }
    for crate_name, expected_dependencies in dependency_sets.items():
        crate_path = root / "crates" / f"jocforge-{crate_name}"
        manifest = tomllib.loads((crate_path / "Cargo.toml").read_text(encoding="utf-8"))
        dependencies = set(manifest.get("dependencies", {}))
        require(dependencies == expected_dependencies, f"dependency boundary drift in {crate_name}")
        require(not (crate_path / "build.rs").exists(), f"unapproved build script in {crate_name}")

        for source_path in (crate_path / "src").rglob("*.rs"):
            source = source_path.read_text(encoding="utf-8")
            production = source.split("#[cfg(test)]", 1)[0]
            lowered = production.lower().replace("deployedcompat", "").replace("deployed_compat", "")
            forbidden_tokens = [
                "legacy",
                "compat",
                "fallback",
                "ffmpeg",
                "concat!(",
                "include!(",
                "#[path",
            ]
            require(
                not any(token in lowered for token in forbidden_tokens),
                f"forbidden production reference in {source_path.relative_to(root)}",
            )
            require(
                not re.search(r"\btrait\b[\s\S]*?\busize\b", production),
                f"trait exposes a raw usize in {source_path.relative_to(root)}",
            )
            require(
                not re.search(
                    r"pub\s+(?:(?:async|unsafe)\s+)*(?:extern\s+\"[^\"]+\"\s+)?fn[\s\S]{0,400}\busize\b",
                    production,
                ),
                f"public API exposes a raw usize in {source_path.relative_to(root)}",
            )
            require(
                not re.search(r"\b(?:async|unsafe)\s+pub\s+fn\b|\bpub\s+unsafe\s+extern\b", production),
                f"unapproved async or unsafe API in {source_path.relative_to(root)}",
            )

    domain_source = root / "crates" / "jocforge-domain" / "src"
    forbidden_io_patterns = [
        r"\bstd::fs\b",
        r"\bstd::process\b",
        r"\bstd::env\b",
        r"\bstd::net\b",
        r"\bFile::open\b",
        r"\bCommand::new\b",
        r"\benv::var\b",
    ]
    for source_path in domain_source.rglob("*.rs"):
        source = source_path.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
        require(
            not any(re.search(pattern, source) for pattern in forbidden_io_patterns),
            f"domain contains file, process, environment, or network I/O: {source_path.relative_to(root)}",
        )

    for module in ("codec", "integrity", "joc"):
        module_root = domain_source / module
        for source_path in module_root.rglob("*.rs"):
            if source_path.name == "mod.rs":
                continue
            code = "\n".join(
                line
                for line in source_path.read_text(encoding="utf-8").splitlines()
                if line.strip() and not line.lstrip().startswith("//")
            )
            require(not code, f"algorithm implementation appeared in empty module {source_path.relative_to(root)}")

    transaction_module = domain_source / "transaction" / "mod.rs"
    transaction_code = "\n".join(
        line
        for line in transaction_module.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.lstrip().startswith("//")
    )
    require(not transaction_code, "callback transaction implementation appeared before its slice")
    return True


def run_checks(root=ROOT, package=PACKAGE_ROOT):
    require(package.is_dir(), f"approved implementation package is unavailable: {package}")
    map_path = root / "docs" / "implementation" / "IMPLEMENTATION-MAP.json"
    data = read_json(map_path)
    require(data.get("schema") == "jocforge-implementation-map.v1", "implementation map schema identity drift")
    schema = read_json(root / "docs" / "implementation" / "IMPLEMENTATION-MAP.schema.json")
    require(schema["properties"]["functional_entries"]["maxItems"] == 104, "map schema no longer pins 104 entries")

    mapping_stats = check_active_mapping(root, package, data)
    check_project_boundaries(root)

    spec = importlib.util.spec_from_file_location("implementation_map_renderer", root / "scripts" / "render-implementation-maps.py")
    require(spec is not None and spec.loader is not None, "cannot load implementation map renderer")
    renderer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(renderer)
    for name, contents in renderer.render(data).items():
        path = root / "docs" / "implementation" / name
        require(path.is_file() and path.read_text(encoding="utf-8") == contents, f"generated map differs: {name}")

    return mapping_stats


def main():
    try:
        stats = run_checks()
    except Exception as error:
        print("boundary check failed: " + str(error), file=sys.stderr)
        return 1
    print(
        "boundary checks passed "
        f"({stats['functional_ids']} functional IDs: {stats['active_ids']} scoped, "
        f"{stats['functional_ids'] - stats['active_ids']} deferred; "
        f"{stats['deferred_roots']} programme roots inventoried)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
