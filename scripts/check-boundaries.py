# pattern: Imperative Shell
"""Explicit architecture and source gates; safe under python -O."""
from __future__ import annotations
import importlib.util, json, pathlib, re, sys, tomllib
class BoundaryError(RuntimeError): pass
ROOT=pathlib.Path(__file__).resolve().parents[1]
PACKAGE=pathlib.Path(r"C:\Users\chyinan\Documents\Codex\2026-10-12\files-pasted-by-the-user-agent\outputs\IMPLEMENTER_PACKAGE")
def check(ok,msg):
    if not ok: raise BoundaryError(msg)
def text(p): check(p.is_file(),f"missing file: {p}"); return p.read_text(encoding="utf-8")
def refs(entry):
    for group in (entry["spec_files"],entry["parameters"]["provider_files"],entry["parameters"]["generation_specs"],entry["parameters"]["validation_files"],entry["vectors"]["authoritative_vectors"],entry["vectors"]["parent_vectors"],entry["vectors"]["review_evidence"],entry["vectors"]["coverage_ledger"]):
        for value in group: yield value
def load_renderer(root):
    spec=importlib.util.spec_from_file_location("map_renderer",root/"scripts/render-implementation-maps.py"); check(spec and spec.loader,"renderer unavailable"); mod=importlib.util.module_from_spec(spec); spec.loader.exec_module(mod); return mod
def source_gate(path,src,empty=False,allow_io=False):
    low=src.lower(); compact=re.sub(r"\s+","",low)
    for word in ("legacy","compat","fallback","ffmpeg","concat!","include!","#[path"):
        check(word not in low,f"forbidden production token {word}: {path}")
    if not allow_io:
        check(not re.search(r"use\s+std(?:::\{[^}]*|::)(?:fs|process|env|net)\b",src,re.S),f"aliased I/O: {path}")
        check(not any(x in compact for x in ("std::fs","std::process","std::env","std::net")),f"production I/O: {path}")
    check("macro_rules!" not in src,f"macro_rules in empty/production boundary: {path}")
    declarations=re.findall(r"(?:pub\s+(?:(?:async|unsafe)\s+)*(?:extern\s+\"[^\"]+\"\s+)?fn|trait\s+\w+)[\s\S]{0,700}?(?:\{|;)",src)
    check(all(not re.search(r"\busize\b",d) for d in declarations),f"public/trait usize: {path}")
    check(not (re.search(r"\btrait\b",src) and re.search(r"\busize\b",src)),f"trait usize: {path}")
    if empty:
        code="\n".join(line for line in src.splitlines() if line.strip() and not line.lstrip().startswith(("//","//!")))
        check(not code or all(re.fullmatch(r"pub mod [a-z0-9_]+;",x.strip()) for x in code.splitlines()),f"empty leaf contains item: {path}")
def run_checks(root=ROOT,package=PACKAGE):
    architecture=root/"docs/architecture"; docs=root/"docs/implementation"
    required_arch={"APPROVAL.md","ARCHITECTURE.md","ARCHITECTURE-DECISIONS.md","DATAFLOW.md","DEPENDENCY-RULES.md","ERROR-MODEL.md","MODULE-MAP.md","OLD-WIRE-REUSE-MATRIX.md","PROFILE-DIMENSIONS.md","PUBLIC-API.md","STATE-OWNERSHIP.md","TEST-STRATEGY.md"}
    check(required_arch <= {p.name for p in architecture.glob("*.md")},"missing architecture document")
    mp=json.loads(text(docs/"IMPLEMENTATION-MAP.json")); schema=json.loads(text(docs/"IMPLEMENTATION-MAP.schema.json"))
    check(mp.get("schema")=="jocforge-implementation-map.v1","map schema identity")
    for key in schema["required"]: check(key in mp,f"schema required: {key}")
    entries=mp["functional_entries"]; check(len(entries)==104,"functional count")
    matrix=json.loads(text(package/"SPEC-COMPLETENESS-MATRIX.json")); expected={x["functional_id"] for x in matrix["modules"] if x["readiness_stage"]=="PRE_HANDOFF_SPEC"}; ids=[x["functional_id"] for x in entries]; check(len(set(ids))==104 and set(ids)==expected,"functional set")
    pk={"FILE_PROVIDER","GENERATED_PROVIDER","COMPOSITE_PROVIDER","NO_RUNTIME_PARAMETER","BLOCKED_MISSING_PROVIDER"}; vk={"DIRECT_VECTOR","PARENT_INTEGRATION_VECTOR","RULE_AND_PROPERTY_TEST","NO_DIRECT_VECTOR","BLOCKED_MISSING_VECTOR"}
    allowed=("crates/jocforge-domain/","crates/jocforge-io/","crates/jocforge-application/","crates/jocforge-validation/")
    byid={x["functional_id"]:x for x in entries}
    for e in entries:
        for key in ("spec_files","implementation_files","test_modules","owner","clock","parameters","vectors","error_outcomes","planned_phase","phase_scope","status"): check(key in e,f"entry field {e['functional_id']}:{key}")
        check(e["implementation_files"] and e["test_modules"],f"empty plan {e['functional_id']}"); check(all(x.startswith(allowed) and x.endswith(".rs") for x in e["implementation_files"]+e["test_modules"]),f"illegal planned path {e['functional_id']}")
        p=e["parameters"]; v=e["vectors"]; check(p["kind"] in pk and v["kind"] in vk,f"kind {e['functional_id']}")
        check(all(x.startswith("parameters/") or x.startswith("test-vectors/") for x in p["provider_files"]),f"provider directory {e['functional_id']}"); check(all("PARAMETER-INTERFACES.json" not in x for x in p["provider_files"]),f"registry provider {e['functional_id']}")
        check(all("PRE-HANDOFF-EVIDENCE-COVERAGE" not in x and not re.search(r"STATUS|\.md$",x,re.I) for x in v["authoritative_vectors"]),f"non-vector direct asset {e['functional_id']}"); check(v["kind"]!="DIRECT_VECTOR" or v["authoritative_vectors"],f"empty direct vector {e['functional_id']}")
        for x in refs(e): check((package/x).is_file(),f"missing mapped asset {e['functional_id']}:{x}")
        for o in e["error_outcomes"]:
            for key in ("error_code","code_authority","trigger","category","staged_state_created","committed_state_changed","candidate_search_continues","programme_state_after","reset_required","caller_action","spec_reference"): check(key in o,f"outcome field {e['functional_id']}:{key}")
            check(not o["committed_state_changed"],f"failure commits {e['functional_id']}"); check((package/o["spec_reference"]["file"]).is_file(),f"outcome spec {e['functional_id']}")
            if o["programme_state_after"]=="RETIRED": check("full restart" in o["caller_action"] or "destroy" in o["caller_action"],f"retired caller action {e['functional_id']}")
    check(byid["F05R56"]["implementation_files"]==["crates/jocforge-domain/src/spatial/reader56.rs"],"F05R56 path")
    check(len(byid["F03N"]["error_outcomes"])>=3 and len(byid["F12"]["error_outcomes"])>=3 and len(byid["GTC"]["error_outcomes"])>=2,"required multi outcomes")
    roots=mp["programme_roots"]; expected_roots={"reader_eof","frontend_control","spatial_control","spatial_audio","joc","meter","drc","analysis","codec","metadata","integrity","output"}; check({x["root_id"] for x in roots}==expected_roots,"programme roots")
    state_doc=text(architecture/"STATE-OWNERSHIP.md"); documented=set(re.findall(r"^\| `([a-z_]+)` \|",state_doc,re.M)); check(expected_roots <= documented,"STATE-OWNERSHIP programme roots differ from machine map")
    for r in roots:
        for key in ("schema_file","schema_pointer","cold_provider_files","generation_specs","canonical_serialization","stage_authority","commit_authority","reset_authority","fatal_behavior","receipt_type","owning_module"): check(r.get(key) not in (None,"",[]),f"root field {r['root_id']}:{key}")
        for x in [r["schema_file"],*r["cold_provider_files"],*r["generation_specs"]]: check((package/x).is_file(),f"root asset {r['root_id']}:{x}")
    overlap=mp["selected_overlap_projection"]; check(overlap["codec_and_publication_root_mapping"]=={"codec_state":"codec","integrity_receipt":"integrity","staging_stream":"output","durable_ledger":"output","published_au_count":"output"},"codec publication split")
    p1=mp["phase_one"]; check(p1["P1.1"]["status"]=="HOLD" and p1["P1.2"]==p1["P1.3"]==p1["P1.4"]=="HOLD","phase holds"); tasks={x["task_id"]:x for x in p1["P1.1"]["tasks"]}; check(set(tasks)=={"CLOCK-FOUNDATION","SELECTED-PROFILE-GATE"},"P1.1 tasks"); check("F01C" not in tasks["CLOCK-FOUNDATION"]["functional_scope"],"F01C in P1.1"); check(tasks["SELECTED-PROFILE-GATE"]["production_dimensions"]==["AuthoredInputProfile","CoreCarrierProfile","JocDownmixProfile","SubstreamTopology","CodecRateProfile","EmdfCarriageProfile"] and tasks["SELECTED-PROFILE-GATE"]["validation_profile"]==["RenderTarget"],"profile separation")
    renderer=load_renderer(root); rendered=renderer.render(mp)
    for name,value in rendered.items(): check(text(docs/name)==value,f"generated Markdown differs: {name}")
    deps={"domain":set(),"application":{"jocforge-domain"},"io":{"jocforge-domain"},"cli":{"jocforge-application"},"validation":{"jocforge-domain","jocforge-application","jocforge-io"}}
    for name,want in deps.items():
        crate=root/"crates"/f"jocforge-{name}"; cfg=tomllib.loads(text(crate/"Cargo.toml")); check(set(cfg.get("dependencies",{}))==want,f"dependency {name}"); check(not (crate/"build.rs").exists(),f"build script {name}")
        for src in (crate/"src").rglob("*.rs"): source_gate(src,text(src),src.name!="mod.rs" and ("codec" in src.parts or "integrity" in src.parts or "joc" in src.parts),name in {"io","cli","validation"})
def main():
    try: run_checks()
    except BoundaryError as e: print(f"boundary check failed: {e}",file=sys.stderr); return 1
    print("boundary checks passed (machine map authoritative; 104 IDs; 12 roots; generated views stable)"); return 0
if __name__=="__main__": raise SystemExit(main())
