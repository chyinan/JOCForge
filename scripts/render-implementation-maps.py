# pattern: Imperative Shell
import argparse,json,pathlib,sys
R=pathlib.Path(__file__).resolve().parents[1];D=R/'docs/implementation';M=D/'IMPLEMENTATION-MAP.json'
def q(x):return f'`{x}`'
def refs(x):return '—'if not x else'<br>'.join(q(v)for v in x)
def tab(t,c,rs):return f'# {t}\n\n由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n| '+' | '.join(c)+' |\n|'+'|'.join('---'for _ in c)+'|\n'+'\n'.join('| '+' | '.join(str(x).replace('|','\\|')for x in r)+' |'for r in rs)+'\n'
def render(d):
 es=d['functional_entries'];o={};o['SPEC-TRACEABILITY.md']=tab('规格追踪',['functional_id','spec_files','owner','planned_phase','inventory_status','implementation_status','implementation_files','test_modules'],[[e['functional_id'],refs(e['spec_files']),q(e['owner']),e['planned_phase'],e['inventory_status'],e['implementation_status'],refs(e.get('implementation_files',[])),refs(e.get('test_modules',[]))]for e in es]);o['OWNER-MAP.md']=tab('Owner 映射',['functional_id','owner','inventory_status'],[[e['functional_id'],q(e['owner']),e['inventory_status']]for e in es]);o['CLOCK-MAP.md']=tab('时钟映射',['functional_id','domains','covered_clauses','inventory_status'],[[e['functional_id'],refs(e.get('clock',{}).get('domains',[])),refs(e.get('clock',{}).get('covered_clauses',[])),e['inventory_status']]for e in es]);er=[]
 for e in es:
  for x in e.get('error_outcomes',[]):er.append([e['functional_id'],q(x['error_code']),x['category'],x['trigger'],x['programme_state_after'],x['caller_action']])
 o['ERROR-MAP.md']=tab('错误 outcome 映射',['functional_id','error_code','category','trigger','programme_state_after','caller_action'],er);o['PARAMETER-MAP.md']=tab('参数映射',['functional_id','kind','provider_files','inventory_status'],[[e['functional_id'],e.get('parameters',{}).get('kind','—'),refs(e.get('parameters',{}).get('provider_files',[])),e['inventory_status']]for e in es]);o['VECTOR-MAP.md']=tab('Vector 映射',['functional_id','kind','authoritative_vectors','case_pointers','inventory_status'],[[e['functional_id'],e.get('vectors',{}).get('kind','—'),refs(e.get('vectors',{}).get('authoritative_vectors',[])),refs(e.get('vectors',{}).get('case_pointers',[])),e['inventory_status']]for e in es]);p=d['phase_one']['P1.1'];o['PHASE-PLAN.md']=tab('Phase 1 计划',['task_id','functional_scope','covered_clauses','planned_files','planned_tests','status'],[[t['task_id'],json.dumps(t.get('functional_scope',{}),ensure_ascii=False,sort_keys=True),refs(t.get('covered_clauses',[])),refs(t.get('planned_files',[])),refs(t.get('planned_tests',[])),t['status']]for t in p['tasks']])+'\n## Failing-first tests\n\n'+'\n'.join(f'{i}. {x}'for i,x in enumerate(p['failing_first_tests'],1))+'\n'
 if 'P1.2-A'in d['phase_one']:
  a=d['phase_one']['P1.2-A'];o['PHASE-PLAN.md']+='\n## P1.2-A consumer-driven numeric slice\n\n```json\n'+json.dumps(a,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.2-B'in d['phase_one']:
  b=d['phase_one']['P1.2-B'];o['PHASE-PLAN.md']+='\n## P1.2-B finite R32 add/multiply slice\n\n```json\n'+json.dumps(b,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.2-C'in d['phase_one']:
  c=d['phase_one']['P1.2-C'];o['PHASE-PLAN.md']+='\n## P1.2-C finite F32 truncation slice\n\n```json\n'+json.dumps(c,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.2-D'in d['phase_one']:
  x=d['phase_one']['P1.2-D'];o['PHASE-PLAN.md']+='\n## P1.2-D normalized Q15 coordinate slice\n\n```json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.2-E'in d['phase_one']:
  x=d['phase_one']['P1.2-E'];o['PHASE-PLAN.md']+='\n## P1.2-E CurrentFeatures finite R32 subtraction primitive\n\n```json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.2-closure'in d['phase_one']:
  x=d['phase_one']['P1.2-closure'];o['PHASE-PLAN.md']+='\n## P1.2 scoped closure\n\n```json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.3-A'in d['phase_one']:
  x=d['phase_one']['P1.3-A'];root=next(r for r in d['programme_roots']if r['root_id']==x['programme_root']);o['PHASE-PLAN.md']+='\n## P1.3-A reader_eof materialization audit\n\n```json\n'+json.dumps({'task':x,'programme_root':root},ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.3-B'in d['phase_one']:
  x=d['phase_one']['P1.3-B'];root=next(r for r in d['programme_roots']if r['root_id']==x['programme_root']);o['PHASE-PLAN.md']+='\n## P1.3-B frontend_control materialization audit\n\n```json\n'+json.dumps({'task':x,'programme_root':root},ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.3-C'in d['phase_one']:
  x=d['phase_one']['P1.3-C'];o['PHASE-PLAN.md']+='\n## P1.3-C two-root callback coordinator mapping\n\n```json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.3-D'in d['phase_one']:
  x=d['phase_one']['P1.3-D'];root=next(r for r in d['programme_roots']if r['root_id']==x['programme_root']);o['PHASE-PLAN.md']+='\n## P1.3-D spatial_control root mapping audit\n\n```json\n'+json.dumps({'task':x,'programme_root':root},ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.3-D'in d['phase_one']:
  x=d['phase_one']['P1.3-D'];o['PHASE-PLAN.md']+='\n## P1.3-D spatial_control root audit\n\n```json\n'+json.dumps({'task':x,'programme_root_reference':f"programme_roots[{x['programme_root']}]"},ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.3-E'in d['phase_one']:
  x=d['phase_one']['P1.3-E'];root=next(r for r in d['programme_roots']if r['root_id']==x['programme_root']);o['PHASE-PLAN.md']+='\n## P1.3-E spatial_audio programme-root mapping\n\n```json\n'+json.dumps({'task':x,'programme_root':root},ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.3-D1'in d['phase_one']:
  x=d['phase_one']['P1.3-D1'];o['PHASE-PLAN.md']+='\n## P1.3-D1 spatial_control staging-only implementation\n\n```json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'SOURCE_FLAGS_PRODUCER'in d['phase_one']:
  x=d['phase_one']['SOURCE_FLAGS_PRODUCER'];o['PHASE-PLAN.md']+='\n## SourceFlags producer\n\n```json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.4-A'in d['phase_one']:
  x=d['phase_one']['P1.4-A'];o['PHASE-PLAN.md']+='\n## P1.4-A F01E provider contract audit\n\n```json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
 if 'P1.4-B'in d['phase_one']:
  x=d['phase_one']['P1.4-B'];o['PHASE-PLAN.md']+='\n## P1.4-B F03 selected target cold provider mapping\n\n'+chr(96)*3+'json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n'+chr(96)*3+'\n'
 if 'P1.4-C'in d['phase_one']:
  x=d['phase_one']['P1.4-C'];producer=d['phase_one'].get('CURRENT_FEATURES_PRODUCER');o['PHASE-PLAN.md']+='\n## P1.4-C CurrentFeatures math provider mapping\n\n'+chr(96)*3+'json\n'+json.dumps({'provider_contracts':x,'current_features_producer':producer},ensure_ascii=False,indent=2,sort_keys=True)+'\n'+chr(96)*3+'\n'
 return o
def main():
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');x=a.parse_args();d=json.loads(M.read_text(encoding='utf-8'));bad=[]
 for n,v in render(d).items():
  p=D/n
  if x.check:
   if not p.is_file()or p.read_text(encoding='utf-8')!=v:bad.append(n)
  else:p.write_text(v,encoding='utf-8')
 if bad:print('generated maps differ: '+', '.join(bad),file=sys.stderr);return 1
 return 0
if __name__=='__main__':raise SystemExit(main())
