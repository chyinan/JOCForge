# pattern: Imperative Shell
"""Check approved architecture, dependency, ownership, and source boundaries."""
import json
import pathlib
import re
import tomllib

root = pathlib.Path(__file__).resolve().parents[1]
package = pathlib.Path(
    r"C:\Users\chyinan\Documents\Codex\2026-10-12\files-pasted-by-the-user-agent"
    r"\outputs\IMPLEMENTER_PACKAGE"
)
expected = {
    "domain": set(),
    "application": {"jocforge-domain"},
    "io": {"jocforge-domain"},
    "cli": {"jocforge-application"},
    "validation": {"jocforge-domain", "jocforge-application", "jocforge-io"},
}
required_architecture = {
    "APPROVAL.md",
    "ARCHITECTURE.md",
    "ARCHITECTURE-DECISIONS.md",
    "DATAFLOW.md",
    "DEPENDENCY-RULES.md",
    "ERROR-MODEL.md",
    "MODULE-MAP.md",
    "OLD-WIRE-REUSE-MATRIX.md",
    "PROFILE-DIMENSIONS.md",
    "PUBLIC-API.md",
    "STATE-OWNERSHIP.md",
    "TEST-STRATEGY.md",
}
required_implementation = {
    "CLOCK-MAP.md",
    "ERROR-MAP.md",
    "OWNER-MAP.md",
    "PARAMETER-MAP.md",
    "PHASE-PLAN.md",
    "SPEC-TRACEABILITY.md",
    "VECTOR-MAP.md",
}
architecture_dir = root / "docs" / "architecture"
implementation_dir = root / "docs" / "implementation"
assert required_architecture <= {path.name for path in architecture_dir.glob("*.md")}
assert required_implementation <= {path.name for path in implementation_dir.glob("*.md")}

workspace = tomllib.loads((root / "Cargo.toml").read_text())
assert set(workspace["workspace"]["members"]) == {
    f"crates/jocforge-{name}" for name in expected
}
assert {p.name for p in (root / "crates").iterdir()} == {
    f"jocforge-{name}" for name in expected
}
for name, dependencies in expected.items():
    crate = root / "crates" / f"jocforge-{name}"
    config = tomllib.loads((crate / "Cargo.toml").read_text())
    assert set(config.get("dependencies", {})) == dependencies, name
    for dependency, values in config.get("dependencies", {}).items():
        assert values == {"path": f"../{dependency}"}, dependency
    assert not config.get("features"), name
    assert not config.get("build-dependencies"), name
    assert not config.get("dev-dependencies"), name
    assert not config.get("target"), name
    assert not (crate / "build.rs").exists(), name
    for source in (crate / "src").rglob("*.rs"):
        text = source.read_text()
        assert not re.search(
            r'include!|#\[path|static\s+mut|legacy|ffmpeg|compat|fallback', text, re.I
        ), source
        if name in {"domain", "application"}:
            assert not re.search(r'\b(fs|process|env|net|thread)\b', text), source
        if name in {"domain", "application"}:
            for public_line in re.findall(r"^\s*pub[^\n]*", text, re.MULTILINE):
                assert not re.search(r"\busize\b", public_line), (source, public_line)
    if name != "validation":
        assert "jocforge-validation" not in config.get("dependencies", {}), name

profile_text = (architecture_dir / "PROFILE-DIMENSIONS.md").read_text()
assert "UNSUPPORTED_PROFILE" in profile_text
assert "AuthoredInputProfile" in profile_text
assert "CoreCarrierProfile" in profile_text
assert "JocDownmixProfile" in profile_text
assert "SubstreamTopology" in profile_text
assert "CodecRateProfile" in profile_text
assert "EmdfCarriageProfile" in profile_text
assert "RenderTarget" in profile_text
supported_rows = [
    line
    for line in profile_text.splitlines()
    if line.startswith("| selected ADM BWF /")
]
assert len(supported_rows) == 1, supported_rows
registered_carriage = set(re.findall(r"`([^`]+)`", supported_rows[0].split("|")[6]))
assert registered_carriage == {"deployed_compat", "etsi_emdf"}, registered_carriage

matrix = json.loads((package / "SPEC-COMPLETENESS-MATRIX.json").read_text())
expected_ids = {
    row["functional_id"]
    for row in matrix["modules"]
    if row["readiness_stage"] == "PRE_HANDOFF_SPEC"
}
assert len(expected_ids) == 104
traceability = (implementation_dir / "SPEC-TRACEABILITY.md").read_text()
trace_rows = [
    line for line in traceability.splitlines() if re.match(r"^\| [A-Z0-9]+ \|", line)
]
assert len(trace_rows) == 104, len(trace_rows)
mapped_ids = set()
for line in trace_rows:
    cells = [cell.strip() for cell in line.strip("|").split("|")]
    assert len(cells) == 9, (len(cells), line)
    functional_id, _, spec_file, parameter_file, vector_file, *_ = cells
    assert functional_id not in mapped_ids, functional_id
    mapped_ids.add(functional_id)
    for relative in (spec_file, parameter_file, vector_file):
        asset = relative.strip("`")
        assert (package / asset).is_file(), (functional_id, asset)
assert mapped_ids == expected_ids, (expected_ids - mapped_ids, mapped_ids - expected_ids)
for map_name in (
    "OWNER-MAP.md",
    "CLOCK-MAP.md",
    "ERROR-MAP.md",
    "PARAMETER-MAP.md",
    "VECTOR-MAP.md",
    "PHASE-PLAN.md",
):
    map_text = (implementation_dir / map_name).read_text()
    map_ids = [
        match.group(1)
        for line in map_text.splitlines()
        if (match := re.match(r"^\| ([A-Z0-9]+) \|", line))
    ]
    assert len(map_ids) == 104, (map_name, len(map_ids))
    assert set(map_ids) == expected_ids, map_name

projection = json.loads(
    (package / "parameters" / "selected-owner-projection-v1.json").read_text()
)
assert len(projection["field_groups"]) == 7
receipt_owners = {group["receipt_owner"] for group in projection["field_groups"]}
commit_owners = {
    group["commit_owner"]
    for group in projection["field_groups"]
}
assert len(receipt_owners) == 7
assert len(commit_owners) == 6
owned_fields = {}
for group in projection["field_groups"]:
    for field in group["fields"]:
        assert field not in owned_fields, (field, owned_fields[field], group["group"])
        owned_fields[field] = group["group"]

empty_leaf_modules = [
    root / "crates/jocforge-domain/src/codec" / name
    for name in (
        "bap.rs",
        "bitio.rs",
        "core_layout.rs",
        "frame_builder.rs",
        "frame_size.rs",
        "frame_syntax.rs",
        "mantissa.rs",
        "rate_profile.rs",
        "substream_topology.rs",
    )
] + [
    root / "crates/jocforge-domain/src/integrity" / name
    for name in ("crc16.rs", "keyless_checksum.rs")
] + [
    root / "crates/jocforge-domain/src/joc" / name
    for name in (
        "coefficient_payload.rs",
        "downmix_profile.rs",
        "emdf.rs",
        "huffman.rs",
        "oamd.rs",
    )
]
for module in empty_leaf_modules:
    text = module.read_text()
    assert text.startswith("// pattern: Functional Core"), module
    assert not re.search(r"^\s*(pub\s+)?(fn|struct|enum|const|static|type)\b", text, re.MULTILINE), module

validation_leaf_modules = [
    root / "crates/jocforge-validation/src" / name / "mod.rs"
    for name in (
        "channel_map_validator",
        "eac3_parser",
        "frame_report",
        "topology_validator",
        "w04",
    )
]
for module in validation_leaf_modules:
    text = module.read_text()
    assert text.startswith("// pattern: Imperative Shell"), module
    assert not re.search(r"^\s*(pub\s+)?(fn|struct|enum|const|static|type)\b", text, re.MULTILINE), module

print(
    "architecture and source boundary checks passed "
    "(12 architecture docs; 7 implementation maps; 104 functional IDs; "
    "7 receipt owners; 6 commit-authority labels; lexical guard)"
)
