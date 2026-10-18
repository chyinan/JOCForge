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
 if 'P1.3-E1'in d['phase_one']:
  x=d['phase_one']['P1.3-E1'];o['PHASE-PLAN.md']+='\n## P1.3-E1 spatial_audio cold-state/staging foundation\n\n```json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n```\n'
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
 if 'P1.4-D1'in d['phase_one']:
  x=d['phase_one']['P1.4-D1'];o['PHASE-PLAN.md']+='\n## P1.4-D1 F04 selected component parameter providers\n\n'+chr(96)*3+'json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n'+chr(96)*3+'\n'
 if 'P1.4-D2'in d['phase_one']:
  x=d['phase_one']['P1.4-D2'];o['PHASE-PLAN.md']+='\n## P1.4-D2 F04 remaining provider validators\n\n'+chr(96)*3+'json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n'+chr(96)*3+'\n'
 if 'QMF_FFT_PRIMITIVE' in d['phase_one']:
  x=d['phase_one']['QMF_FFT_PRIMITIVE'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  o['PHASE-PLAN.md']+='\n## QMF FFT primitive mapping\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['QMF-FFT-PRIMITIVE-MAP.md']='# QMF FFT primitive map\n\n由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'+f"Status: `{x['status']}`. This covers only the fixed QMF FFT primitive; QMF callers and whole-chain execution remain separate.\n\n"+'The primitive is a 64-point complex transform over 128 interleaved F32 scalar slots. The map keeps input normalization and caller reorder/sign-bit operations outside the butterfly contract.\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
 if 'QMF_CORE' in d['phase_one']:
  x=d['phase_one']['QMF_CORE'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  o['PHASE-PLAN.md']+='\n## QMF analysis/synthesis core mapping\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['QMF-ANALYSIS-SYNTHESIS-CORE-MAP.md']=(
   '# QMF analysis/synthesis core map\n\n'
   '由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'
   +f"Status: `{x['status']}`. This is a docs-only readiness gate for the selected fixed QMF core; it does not promote the F04 whole root.\n\n"
   +'## Processing boundary\n\n'
   +'Analysis accepts one 64-sample lane slot, that lane’s nine-row history, the selected prototype and rotation, and the fixed FFT provider. The caller has already produced `s09_input`; lane routing and receiver preprocessing stay outside this core.\n\n'
   +'The receiver mode=1 path applies the ten-slot delay and band0 FIR only to supplied surround lanes. It then performs the inverse reorder, unscaled FFT, postmodulation, and synthesis polyphase step. The core returns after-state slices; only `spatial_audio` may commit them later.\n\n'
   +'## State and clocks\n\n'
   +'`ModelState.analysis_history` owns five independent histories in `[L,R,C,Ls,Rs]` order. Receiver mode=1 delay, FIR, and synthesis histories are two independent rows for `[Ls,Rs]`. `QmfWindowState.fitting_delay/fitting_fir` remain separate and mode=0 fitting is excluded.\n\n'
   +'The parent callback uses `FrontendCallbackIndex g`. The receiver uses its separate `ReceiverCallbackCount r`: no receiver/QMF call for `g<2`, then `r=g-2` and `receiver_generation=r+1`. Each receiver callback processes 24 ordered QMF slots; EOF adds no synthetic QMF flush.\n\n'
   +'## Provider and vector evidence\n\n'
   +'The qmf functional provider supplies the selected QWIN and mode=1 band0 taps. The mathematical FFT provider supplies the 63 roots and separate 128-word rotation. `receiver_blend32` is an upstream F04R.5 dependency; QMF does not consume it directly. The current fresh review is static only. Direct unit vectors cover FFT, reorder, synthesis history and surround delay/FIR, while fold/rotation and postmodulation lack isolated direct cases; model-first whole-binding hashes remain assertions, not execution evidence for this implementation.\n\n'
   +'The machine-readable record below binds the exact formulas, providers, owners, clocks, vectors and exclusions.\n\n'
   +chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  )
 if 'RECEIVER_MODE1_ROUTING' in d['phase_one']:
  x=d['phase_one']['RECEIVER_MODE1_ROUTING'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  o['PHASE-PLAN.md']+='\n## Receiver mode=1 routing and state audit\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['RECEIVER-MODE1-MAP.md']=(
   '# Receiver mode=1 route and state map\n\n'
   +'由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'
   +f"Mapping status: `{x['mapping_status']}`. Implementation readiness: `{x['implementation_readiness']}`.\n\n"
   +'All five carriers `[L,R,C,Ls,Rs]` run 24-slot analysis on receiver lanes `[0,1,2,4,5]`. Only surround lanes `[4,5]` run mode=1 delay/FIR/synthesis. L/R/C update analysis history only in this audio-QMF path; lane 3 LFE bypasses QMF.\n\n'
   +'Frontend callback `g` remains distinct from receiver count `r`: `g<2` is Type10 priming and does not advance receiver/QMF; for `g>=2`, `r=g-2`, receiver after-generation is `r+1=g-1`, and JOC dispatch is `d=g-1`. The callback count is proposed only with the complete `spatial_audio` root.\n\n'
   +'The route and owner fields are mapped. Implementation readiness remains blocked because the supplied model-first review statuses pin four files to hashes that differ from the current package manifest. The blocker lists the exact identities.\n\n'
   +'## Machine-readable contract\n\n'
   +chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  )
 if 'POSTWINDOW_FANOUT_FOUNDATION' in d['phase_one']:
  x=d['phase_one']['POSTWINDOW_FANOUT_FOUNDATION'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  o['PHASE-PLAN.md']+='\n## PostWindow fanout foundation\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['POSTWINDOW-FANOUT-MAP.md']='# PostWindow fanout map\n\n由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'+f"Status: `{x['status']}`. The render producer remains deferred, as do matrix/destination production, F04 assembly, and the parent transaction.\n\n"+'The linear proposal carries one immutable Arc payload. Its two consumer views share that allocation and carry an explicit callback phase.\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
 if 'P1.3-E3' in d['phase_one']:
  x=d['phase_one']['P1.3-E3'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  o['PHASE-PLAN.md']+='\n## P1.3-E3 Direct-LFE ring stage\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['DIRECT-LFE-RING-MAP.md']='# Direct-LFE ring map\n\n由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'+f"Mapping status: `{x['mapping_status']}`. Implementation status: `{x['implementation_status']}`. The full F04 chain and parent transaction remain blocked.\n\n"+'This is only the direct-LFE ring transition and a future Type10 row0 input; it does not construct Type10 or execute receiver/QMF.\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
 if 'P1.3-E4' in d['phase_one']:
  x=d['phase_one']['P1.3-E4'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  o['PHASE-PLAN.md']+='\n## P1.3-E4 Type10 assembly boundary\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['TYPE10-ASSEMBLY-MAP.md']='# Type10 assembly map\n\n由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'+f"Mapping status: `{x['mapping_status']}`. Implementation status: `{x['implementation_status']}`. Receiver/QMF and parent transaction remain deferred.\n\n"+'This slice only assembles the E2 component proposal with the E3 delayed LFE row; it does not execute receiver/QMF or commit spatial_audio state.\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
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
