# pattern: Imperative Shell
"""Render deterministic Markdown views from IMPLEMENTATION-MAP.json."""
from __future__ import annotations
import argparse, json, pathlib, sys

ROOT=pathlib.Path(__file__).resolve().parents[1]; DOCS=ROOT/"docs"/"implementation"; MAP=DOCS/"IMPLEMENTATION-MAP.json"
def q(x): return f"`{x}`"
def refs(xs): return "—" if not xs else "<br>".join(q(x) for x in xs)
def table(title, intro, columns, rows):
    return f"# {title}\n\n{intro}\n\n| "+" | ".join(columns)+" |\n|"+"|".join("---" for _ in columns)+"|\n"+"\n".join("| "+" | ".join(str(x).replace("|","\\|") for x in row)+" |" for row in rows)+"\n"
def render(data):
    es=data["functional_entries"]
    out={}
    out["SPEC-TRACEABILITY.md"]=table("规格追踪","本文件由 IMPLEMENTATION-MAP.json 生成，不得手工修改。",["functional_id","spec_files","implementation_files","test_modules","owner","clock_domains","parameter_kind","parameter_sources","vector_kind","vector_sources","review_evidence","parent_vectors","planned_phase","status"],[[e["functional_id"],refs(e["spec_files"]),refs(e["implementation_files"]),refs(e["test_modules"]),q(e["owner"]),refs(e["clock"]["domains"]),e["parameters"]["kind"],refs(e["parameters"]["provider_files"]+e["parameters"]["generation_specs"]),e["vectors"]["kind"],refs(e["vectors"]["authoritative_vectors"]),refs(e["vectors"]["review_evidence"]),refs(e["vectors"]["parent_vectors"]),e["planned_phase"],e["status"]] for e in es])
    out["OWNER-MAP.md"]=table("Owner 映射","Heavyweight receipt 只用于 committed 或明确 staged owner boundary。",["functional_id","owner","implementation_files"],[[e["functional_id"],q(e["owner"]),refs(e["implementation_files"])] for e in es])
    out["CLOCK-MAP.md"]=table("时钟映射","covered_clauses 是该 ID 明确覆盖的 scoped 条款。",["functional_id","domains","covered_clauses"],[[e["functional_id"],refs(e["clock"]["domains"]),refs(e["clock"]["covered_clauses"])] for e in es])
    erows=[]
    for e in es:
        for o in e["error_outcomes"]:
            erows.append([e["functional_id"],q(o["error_code"]),o["code_authority"],o["trigger"],o["category"],str(o["staged_state_created"]).lower(),str(o["committed_state_changed"]).lower(),str(o["candidate_search_continues"]).lower(),o["programme_state_after"],o["reset_required"],o["caller_action"],q(o["spec_reference"]["file"]),o["spec_reference"]["section"]])
    out["ERROR-MAP.md"]=table("错误 outcome 映射","一个 functional ID 可以有多个语义 outcome。",["functional_id","error_code","code_authority","trigger","category","staged_state_created","committed_state_changed","candidate_search_continues","programme_state_after","reset_required","caller_action","spec_file","spec_section"],erows)
    out["PARAMETER-MAP.md"]=table("参数映射","PARAMETER-INTERFACES.json 只允许作为 registry entry。",["functional_id","kind","provider_files","generation_specs","registry_entries","validation_files","runtime_loader"],[[e["functional_id"],e["parameters"]["kind"],refs(e["parameters"]["provider_files"]),refs(e["parameters"]["generation_specs"]),refs(e["parameters"]["registry_entries"]),refs(e["parameters"]["validation_files"]),"—" if e["parameters"]["runtime_loader"] is None else q(e["parameters"]["runtime_loader"])] for e in es])
    out["VECTOR-MAP.md"]=table("Vector 映射","Coverage ledger 不是 authoritative vector。",["functional_id","kind","authoritative_vectors","parent_vectors","review_evidence","coverage_ledger","test_modules"],[[e["functional_id"],e["vectors"]["kind"],refs(e["vectors"]["authoritative_vectors"]),refs(e["vectors"]["parent_vectors"]),refs(e["vectors"]["review_evidence"]),refs(e["vectors"]["coverage_ledger"]),refs(e["test_modules"])] for e in es])
    p=data["phase_one"]["P1.1"]; rows=[]
    for task in p["tasks"]: rows.append([task["task_id"],json.dumps(task.get("functional_scope",{}),ensure_ascii=False,sort_keys=True),refs(task.get("covered_clauses",[])),refs(task.get("production_dimensions",[])),refs(task.get("validation_profile",[])),p["status"]])
    out["PHASE-PLAN.md"]=table("Phase 1 计划","P1.1 仅含两个 architecture/foundation task；P1.2～P1.4 保持 HOLD。",["task_id","functional_scope","covered_clauses","production_dimensions","validation_profile","status"],rows)+"\n## P1.1 failing-first tests\n\n"+"\n".join(f"{i}. {x}" for i,x in enumerate(p["failing_first_tests"],1))+"\n\n## 后续 slice\n\n- P1.2：HOLD。\n- P1.3：HOLD。\n- P1.4：HOLD。\n"
    return out
def main():
    ap=argparse.ArgumentParser(); ap.add_argument("--check",action="store_true"); args=ap.parse_args(); data=json.loads(MAP.read_text(encoding="utf-8")); rendered=render(data); bad=[]
    for name,text in rendered.items():
        path=DOCS/name
        if args.check:
            if not path.is_file() or path.read_text(encoding="utf-8")!=text: bad.append(name)
        else: path.write_text(text,encoding="utf-8")
    if bad: print("generated maps differ: "+", ".join(bad),file=sys.stderr); return 1
    return 0
if __name__=="__main__": raise SystemExit(main())
