# pattern: Imperative Shell
import argparse,json,pathlib,sys
R=pathlib.Path(__file__).resolve().parents[1];D=R/'docs/implementation';M=D/'IMPLEMENTATION-MAP.json'
def q(x):return f'`{x}`'
def refs(x):return '—'if not x else'<br>'.join(q(v)for v in x)
def tab(t,c,rs):return f'# {t}\n\n由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n| '+' | '.join(c)+' |\n|'+'|'.join('---'for _ in c)+'|\n'+'\n'.join('| '+' | '.join(str(x).replace('|','\\|')for x in r)+' |'for r in rs)+'\n'
def appendix(t,c,rs):return '' if not rs else '\n'+tab(t,c,rs).replace(f'# {t}\n',f'## {t}\n',1)
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
 if 'SOURCE_INVENTORY_EVOLUTION'in d['phase_one']:
  x=d['phase_one']['SOURCE_INVENTORY_EVOLUTION'];o['PHASE-PLAN.md']+='\n## Current source inventory evolution\n\n'+chr(96)*3+'json\n'+json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)+'\n'+chr(96)*3+'\n'
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
   +'Fresh review R1 is bound to the current package and passes 44/44 static checks. Local synthetic tests verify the receiver routing/state stage. The older review mismatches remain recorded as stale identities. F04R production, same-runner F05R56 targets, J05 projection, parent transaction and product acceptance remain NOT_RUN.\n\n'
   +'## Machine-readable contract\n\n'
   +chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  )
 if 'DESCRIPTOR_CONTROL_AUDIT' in d['phase_one']:
  x=d['phase_one']['DESCRIPTOR_CONTROL_AUDIT'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  implementation_summary=(
   'The two-phase F04D stage is locally verified against six selected-provider direct cases and eight error cases. It compares 55,296 lane-major ramp output words plus control/ramp traces, complete after-state, and canonical hold bytes. The pre-temporal3 selection is a non-Clone result consumed by the post-temporal3 gain input; its after-state and s09_return are proposals only, with no local commit or receipt. Whole receiver, J05/F05JC, real ADM/listening, and W04 execution remain `NOT_RUN`.'
   if x['implementation_status'].startswith('LOCALLY_VERIFIED_') else
   'The approved `f04-descriptor-control.v1` supplement binds six selected-provider success vectors, eight error cases, semantic U32-to-I64LE hold encoding, and six local outcomes. This mapping gate is implementation-ready only for the two-phase F04D descriptor-control slice. Whole receiver, J05/F05JC, real ADM/listening, and W04 execution remain `NOT_RUN`.'
  )
  o['PHASE-PLAN.md']+='\n## F04D descriptor control audit\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['DESCRIPTOR-CONTROL-MAP.md']=(
   '# Descriptor control map\n\n'
   +'由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'
   +f"Mapping status: `{x['mapping_status']}`. Implementation readiness: `{x['implementation_readiness']}`.\n\n"
   +'F04D is a receiver substage owned by `spatial_audio`. It selects descriptor control from the post-temporal0..2 scratch before temporal3, then applies the selected gain to six lanes after temporal3. The mode=1 QMF path, mode=0 fitting state, final-six tail, and selected-J05 projection remain separate consumers and owners.\n\n'
   +'P1.4-D2 validates provider identity and typed values only. The selected F04D lookup threshold comes from the QMF receiver functional provider; the mathematical threshold in the descriptor provider is a candidate that differs in 36 of 127 words and remains pending audio acceptance. No descriptor DSP or W04 audio acceptance is claimed.\n\n'
   +implementation_summary+'\n\n'
   +'## Machine-readable contract\n\n'
   +chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  )
 if 'MODE0_FITTING_AUDIT' in d['phase_one']:
  x=d['phase_one']['MODE0_FITTING_AUDIT'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  o['PHASE-PLAN.md']+='\n## F04RB mode=0 fitting mapping audit\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['MODE0-FITTING-MAP.md']=(
   '# Mode=0 QMF fitting map\n\n'
   +'由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'
   +f"Mapping status: {x['mapping_status']}. Implementation status: {x['implementation_status']}. Implementation readiness: {x['implementation_readiness']}.\n\n"
   +'This map authorizes only the F04RB mode=0 fitting stage. The receiver route, F05JC fit, J05/F05JL, final-six tail, and parent transaction remain separate and deferred.\n\n'
   +'The carrier path re-analyzes same-generation s09_input in [L,R,C,Ls,Rs] order from the receiver analysis before-state. Its analysis after-state must equal the receiver analysis after-state wordwise, but the fitting branch proposes only its own delay/FIR/window after-state; it has no local commit authority. Mode=0 updates delay and the 20-complex history while skipping phase rotation, tap multiplication, band0 replacement, and synthesis.\n\n'
   +'Target and carrier windows each join their own previous 24 slots to the current 24 slots. F05JC may consume the resulting 15x48 target and 5x48 carrier only when eligibility, weights, geometry, and the descriptor after index are same-generation ready. s09_return and final_six are not carrier inputs.\n\n'
   +'F04RB-MODE0-FITTING-V1 binds cold g2, warm g3, and cached-drain g4 recipes to post-computation literals and complete canonical after-state hashes. Execute all stages from recipes and prior computed state before comparing literals. The g4 drain repeats g3 input, but still advances the fitting state once. U-QMF-SURROUND-RESET remains a mode=1 case. Existing whole-binding assertions remain model-first and do not qualify whole receiver execution.\n\n'
   +'The API precedence is provider availability, provider identity/shape/finite, callback and projection lineage, before-state, input shape/finite, then numeric or receiver-analysis join. The API has no separate receiver-analysis join code, so this map uses the clean local F04RB_LINEAGE_ERROR outcome; it makes no native error-code claim.\n\n'
   +'The full provider, state, clock, outcome, KAT, and implementation-scope record follows.\n\n'
   +'## Machine-readable contract\n\n'
   +chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  )
 if 'F04R_PREPROCESSING_FINAL_SIX_AUDIT' in d['phase_one']:
  x=d['phase_one']['F04R_PREPROCESSING_FINAL_SIX_AUDIT'];block=json.dumps(x,ensure_ascii=False,indent=2,sort_keys=True)
  def matrix(columns,rows):return '| '+' | '.join(columns)+' |\n|'+'|'.join('---'for _ in columns)+'|\n'+'\n'.join('| '+' | '.join(str(cell).replace('|','\\|')for cell in row)+' |'for row in rows)+'\n'
  o['PHASE-PLAN.md']+='\n## F04R preprocessing and final-six mapping audit\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  lane_rows=[[i,lane,x['lane_contract'][key]]for i,(lane,key)in enumerate(zip(x['lane_contract']['output_order'],('front','front','center','lfe','surround','surround')))]
  pipeline_rows=[[item['order'],item['stage'],item['owner'],item['input_output'],item['state_fields']]for item in x['pipeline_order']]
  delay_rows=[[name,rule]for name,rule in x['delay_and_ring_rules'].items()]
  state_rows=[[item['field'],item['schema'],item['cold']]for item in x['state_ownership']['fields']]
  temporal_serialization=x['state_ownership']['temporal_state_serialization']
  temporal_field_rows=[[item['name'],item['encoding']]for item in temporal_serialization['entry_fields']]
  temporal_instance_rows=[[item['name'],item['span'],item['lanes'],item['buffer_bytes_per_lane'],item.get('route_status','consumed') ]for item in temporal_serialization['instances']]
  provider_rows=[[item['file'],item['bytes'],item['sha256'],item['fields']]for item in x['provider_bindings']]
  vector_rows=[[item['stage'],item['direct_cases'],item['classification']]for item in x['vector_coverage']['stage_coverage']]
  clock_rows=[[name,value]for name,value in x['clock_contract'].items()]
  owner_rows=[[name,value]for name,value in x['owner_and_consumer_boundaries'].items()]
  blocker_rows=[[item['id'],item['reason'],item['minimum_evidence']]for item in x['blockers']]
  o['RECEIVER-PIPELINE-MAP.md']=(
   '# F04R receiver pipeline and final-six map\n\n'
   +'由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'
   +f"Mapping status: `{x['mapping_status']}`. Implementation readiness: `{x['implementation_readiness']}`. Audit status: `{x['status']}`.\n\n"
   +'This is a mapping-only gate. It does not implement Rust/DSP, JOC/J05, F05JC, codec behavior, a parent transaction, or any root commit. The F04R route is executed once by F04C for each callback whose Type10 outcome is Ready.\n\n'
   +'## Callback and lanes\n\n'
   +f"The receiver consumes `{x['type10_input']['shape']}` from the E4 packet/receipt pair for `g>=2`. {x['type10_input']['row_order']} {x['type10_input']['target_updates']}\n\n"
   +matrix(['output lane','identity','route'],lane_rows)+'\n'
   +'## Ordered callback pipeline\n\n'
   +matrix(['order','stage','authority','input -> output','state fields'],pipeline_rows)+'\n'
   +'## Delay, temporal, and ring transitions\n\n'+matrix(['transition','exact rule'],delay_rows)+'\n'
   +'## Persistent ModelState owner\n\n'
   +f"Owner: `{x['state_ownership']['owner']}`. Canonical schema: `{x['state_ownership']['canonical_schema_file']}` ({x['state_ownership']['canonical_schema_bytes']} bytes, SHA-256 `{x['state_ownership']['canonical_schema_sha256']}`), pointer `{x['state_ownership']['schema_pointer']}`.\n\n"
   +matrix(['field','canonical schema / dimension summary','cold/reset'],state_rows)+'\n'
   +f"TemporalState canonical schema pointer: `{temporal_serialization['schema_pointer']}`.\n\n"
   +matrix(['TemporalState field','canonical encoding'],temporal_field_rows)+'\n'
   +matrix(['instance','span','lanes','buffer bytes per lane','route'],temporal_instance_rows)+'\n'
   +f"Semantic cursor crosswalk: `{temporal_serialization['semantic_crosswalk_status']}`. {temporal_serialization['semantic_crosswalk_gap']}\n\n"
   +'The full ModelState is staged as one after-state. F04R/F04D/F04RB have no local commit; only the enclosing F04C/spatial_audio transaction may commit it.\n\n'
   +'## Clocks and drain\n\n'+matrix(['clock','contract'],clock_rows)+'\n'
   +'## F04C/F04D/F04RB and downstream boundaries\n\n'+matrix(['owner or consumer','boundary'],owner_rows)+'\n'
   +'## Provider and vector evidence\n\n'+matrix(['provider','bytes','SHA-256','bound fields'],provider_rows)+'\n'
   +matrix(['stage','direct coverage','qualification'],vector_rows)+'\n'
   +'The six whole-chain vectors remain model-first assertions only and do not qualify the isolated stage. The separate continuous direct KAT computes case1 -> case2 -> case3 from actual after-state, compares intermediate outputs and the complete canonical ModelState after bytes, and is locally verified. The mapped error outcomes, including competing-fault precedence, are also locally verified. Selected provider values are identity-pinned and present.\n\n'
   +'## Minimum blockers\n\n'+matrix(['blocker','gap','minimum evidence'],blocker_rows)+'\n'
   +'F04R-specific error labels and first-failure precedence are mapped by the approved direct receiver contract and locally verified; do not copy F04D, F04RB, or mode=1 error contracts into F04R. The remaining integration boundary is same-runner F05R56 target production, which is `NOT_RUN`. Whole receiver integration, J05/F05JC/parent commit, W04, real ADM, and product acceptance also remain `NOT_RUN`.\n\n'
   +'## Machine-readable contract\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
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
  receiver=d['phase_one'].get('RECEIVER_MODE1_ROUTING',{});receiver_status=receiver.get('status','NOT_MAPPED')
  o['TYPE10-ASSEMBLY-MAP.md']='# Type10 assembly map\n\n由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'+f"Mapping status: `{x['mapping_status']}`. Implementation status: `{x['implementation_status']}`. Receiver mode=1 routing/state status: `{receiver_status}`; whole F04 receiver, J05 projection and parent transaction remain deferred.\n\n"+'This E4 slice only assembles the E2 component proposal with the E3 delayed LFE row; it does not invoke the separate receiver mode=1 stage or commit spatial_audio state.\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
 f05r56=next((e for e in d['functional_entries'] if e['functional_id']=='F05R56'),None)
 if f05r56 and f05r56.get('mapping_status'):
  block=json.dumps(f05r56,ensure_ascii=False,indent=2,sort_keys=True)
  o['PHASE-PLAN.md']+='\n## Reader target updates mapping and implementation gate\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  o['READER-TARGET-UPDATES-MAP.md']=(
   '# Reader target updates map\n\n'
   +'由 IMPLEMENTATION-MAP.json 生成，不得手工修改。\n\n'
   +f"Mapping status: `{f05r56['mapping_status']}`. Implementation status: `{f05r56['implementation_status']}`.\n\n"
   +'F05R56 previews marker/history, the selected Type11 encode/decode boundary, 16 reader56 records, and the 16x6 target payload. The persistent state is exactly the nine fields in `reader56-target-state-schema.v2`; targets and receipts are callback-local and contribute zero canonical state bytes.\n\n'
   +'The callback clock is `FrontendCallbackIndex g`; callbacks 0 and 1 advance marker/history and target snapshot only. For `g>=2`, the separate `ReaderInvocationIndex q` is `g-2`; delivered row generations remain attached per row. The stage returns a proposal and a pending immutable `ReaderTargetUpdateReceipt`; only the enclosing `spatial_control` transaction may commit state and admit that receipt.\n\n'
   +'The selected 8-callback producer chain, six reader invocations, Type11 roundtrips, 96 reader records, 576 target words, and receipt semantic hashes are locally verified against input-only recipes and post-computation assertions. Receipt consumers may use only the admitted same-generation receipt. F05JC/J05, F05L queue/body production, parent transaction implementation, whole-programme execution, and product acceptance remain `NOT_RUN`.\n\n'
   +'## Machine-readable contract\n\n'+chr(96)*3+'json\n'+block+'\n'+chr(96)*3+'\n'
  )
 direct_slices=[(e,s)for e in es for s in e.get('direct_slices',[])]
 if direct_slices:
  label=lambda e,s:f"{e['functional_id']}/{s['slice_id']}"
  local_fixture=lambda s: '—' if not s.get('vectors',{}).get('local_fixture') else q(s['vectors']['local_fixture']['file'])+'<br>'+s['vectors']['local_fixture']['sha256']
  o['SPEC-TRACEABILITY.md']+=appendix('Materialized direct sub-slices; parent functional IDs remain deferred',['slice_id','parent_functional_id','planned_phase','mapping_status','implementation_status','scope','implementation_files','test_modules'],[[label(e,s),e['functional_id'],e['planned_phase'],s['mapping_status'],s['implementation_status'],s['scope'],refs(s['implementation_files']),refs(s['test_modules'])]for e,s in direct_slices])
  o['OWNER-MAP.md']+=appendix('Direct sub-slice owner',['slice_id','parent_functional_id','owner'],[[label(e,s),e['functional_id'],q(s.get('owner',e['owner']))]for e,s in direct_slices])
  o['CLOCK-MAP.md']+=appendix('Direct sub-slice clocks',['slice_id','domains','covered_clauses'],[[label(e,s),refs(s['clock']['domains']),refs(s['clock']['covered_clauses'])]for e,s in direct_slices])
  o['ERROR-MAP.md']+=appendix('Direct sub-slice local outcomes',['slice_id','error_code','category','trigger','programme_state_after / result','caller_action'],[[label(e,s),q(x.get('error_code','—')),x.get('category','—'),x.get('trigger','—'),x.get('programme_state_after',x.get('result','—')),x.get('caller_action','—')]for e,s in direct_slices for x in s.get('error_outcomes',[])])
  o['PARAMETER-MAP.md']+=appendix('Direct sub-slice parameter bindings',['slice_id','kind','provider_files','validation_boundary'],[[label(e,s),s['parameters']['kind'],refs(s['parameters']['provider_files']),s['parameters']['validation_boundary']]for e,s in direct_slices])
  o['VECTOR-MAP.md']+=appendix('Direct sub-slice vectors',['slice_id','kind','authoritative_vectors','case_pointers','local_fixture'],[[label(e,s),s['vectors']['kind'],refs(s['vectors']['authoritative_vectors']),refs(s['vectors']['case_pointers']),local_fixture(s)]for e,s in direct_slices])
  o['PHASE-PLAN.md']+=appendix('Materialized non-Phase-1 direct sub-slices',['slice_id','planned_phase','scope','implementation_files','test_modules','status'],[[label(e,s),e['planned_phase'],s['scope'],refs(s['implementation_files']),refs(s['test_modules']),s['implementation_status']]for e,s in direct_slices])
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
