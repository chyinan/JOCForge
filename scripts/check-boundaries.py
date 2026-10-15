# pattern: Imperative Shell
import importlib.util,json,pathlib,re,sys,tomllib
class BoundaryError(RuntimeError):pass
ROOT=pathlib.Path(__file__).resolve().parents[1];PKG=pathlib.Path(r'C:\Users\chyinan\Documents\Codex\2026-10-12\files-pasted-by-the-user-agent\outputs\IMPLEMENTER_PACKAGE')
PACKAGE=PKG
def ck(x,m):
 if not x:raise BoundaryError(m)
def run(root=ROOT):
 arch=root/'docs/architecture';req={'APPROVAL.md','ARCHITECTURE.md','ARCHITECTURE-DECISIONS.md','DATAFLOW.md','DEPENDENCY-RULES.md','ERROR-MODEL.md','MODULE-MAP.md','OLD-WIRE-REUSE-MATRIX.md','PROFILE-DIMENSIONS.md','PUBLIC-API.md','STATE-OWNERSHIP.md','TEST-STRATEGY.md'};ck(req<={p.name for p in arch.glob('*.md')},'architecture documents')
 d=json.loads((root/'docs/implementation/IMPLEMENTATION-MAP.json').read_text(encoding='utf-8'));schema=json.loads((root/'docs/implementation/IMPLEMENTATION-MAP.schema.json').read_text(encoding='utf-8'));ck(all(k in d for k in schema['required']),'schema required');es=d['functional_entries'];ck(len(es)==104 and len({e['functional_id']for e in es})==104 and all(e['inventory_status']=='INVENTORIED'for e in es),'104 inventoried IDs');active={e['functional_id']for e in es if e.get('active_slice_materialization')=='MATERIALIZED'};ck(active=={'F02','XSTJ'},'active IDs');future=[e for e in es if e['functional_id']not in active];ck(len(future)==102,'deferred count')
 for e in future:
  ck(e['implementation_status']=='DEFERRED_DETAILED_MAPPING','future status');ck(not any(k in e for k in('implementation_files','parameters','vectors','error_outcomes','clock')),'future fake detail')
 clauses=['F01/F02 input sample-count clauses','callback g','codec source s=g-2','JOC dispatch d=s+1=g-1','generated JOC writer=d-1=s','source0/1 suppression','source s>=2 to published AU=s-2','published AU ordinary/JOC writer offsets','programme count arithmetic']
 for e in es:
  for f in e['spec_files']:ck((PKG/f).is_file(),'spec')
  if e['functional_id']in active:ck(e['clock']['covered_clauses']==clauses and e['clock']['domains']==['AdmSampleCount','NormalCallbackCount','FrontendCallbackIndex','CodecSourceIndex','JocDispatchIndex','JocGeneratedWriterIndex','OrdinaryPayloadWriterIndex','JocPoppedWriterIndex','PublishedAuIndex']and e['implementation_files']==['crates/jocforge-domain/src/timeline/index.rs','crates/jocforge-domain/src/timeline/clock.rs','crates/jocforge-domain/src/timeline/mod.rs']and e['test_modules']and e['vectors']['kind']=='RULE_AND_PROPERTY_TEST'and e['error_outcomes'],'active materialization')
 ck(all(r=={'root_id':r['root_id'],'materialization_status':'INVENTORIED_PENDING_P1.3_MATERIALIZATION'}for r in d['programme_roots']),'root precision claim');ck(d['phase_one']['P1.1']['completed_functional_ids']==[],'completed IDs');ck(d['status']['TRACEABILITY']=='PARTIALLY_MATERIALIZED','trace status')
 gate=d['phase_one']['P1.1']['tasks'][1];ck(gate['task_id']=='SELECTED-PROFILE-GATE'and gate['status']=='LOCALLY_VERIFIED'and gate['materialization_status']=='MATERIALIZED','selected gate status');ck(gate['production_dimensions']==['AuthoredInputProfile','CoreCarrierProfile','JocDownmixProfile','SubstreamTopology','CodecRateProfile','EmdfCarriageProfile']and gate['validation_profile']==['RenderTarget'],'profile separation');ck(gate['error_outcome']=={'error_code':'UNSUPPORTED_PROFILE','code_authority':'PACKAGE_NORMATIVE','fallback':False},'profile error/fallback');ck(gate['default_carriage']=='deployed_compat'and gate['explicit_carriage_modes']==['deployed_compat','etsi_emdf']and gate['resolved_profile_accessor']=='carriage','carriage default/accessor');ck(d['phase_one']['P1.2']==d['phase_one']['P1.3']==d['phase_one']['P1.4']=='HOLD','later slices hold')
 sp=importlib.util.spec_from_file_location('renderer',root/'scripts/render-implementation-maps.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
 for n,v in m.render(d).items():ck((root/'docs/implementation'/n).read_text(encoding='utf-8')==v,'render diff '+n)
 ck('RenderTarget'not in json.dumps(d['phase_one']['P1.1']['tasks'][0]),'render target clock');profile_doc=(arch/'PROFILE-DIMENSIONS.md').read_text(encoding='utf-8');ck(len([x for x in profile_doc.splitlines()if x.startswith('| selected ADM BWF /')])==1,'supported row count');production=(root/'crates/jocforge-domain/src/profile/production.rs').read_text(encoding='utf-8');ck('RenderTarget'not in production and 'fallback'not in production.lower()and 'Self::selected_with_carriage(EmdfCarriageProfile::DeployedCompat)'in production,'production profile separation/default')
 deps={'domain':set(),'application':{'jocforge-domain'},'io':{'jocforge-domain'},'cli':{'jocforge-application'},'validation':{'jocforge-domain','jocforge-application','jocforge-io'}}
 for n,w in deps.items():
  c=root/'crates'/f'jocforge-{n}';cfg=tomllib.loads((c/'Cargo.toml').read_text(encoding='utf-8'));ck(set(cfg.get('dependencies',{}))==w,'dependency '+n);ck(not(c/'build.rs').exists(),'build script')
  for p in(c/'src').rglob('*.rs'):
   s=p.read_text(encoding='utf-8').lower();guard=s.replace('deployedcompat','');ck(not any(x in guard for x in('legacy','compat','fallback','ffmpeg','concat!','include!','#[path')),'source token '+str(p));ck(not(re.search(r'\btrait\b',s)and re.search(r'\busize\b',s)),'trait usize');ck(not re.search(r'pub\s+(?:(?:async|unsafe)\s+)*(?:extern\s+"[^"]+"\s+)?fn[\s\S]{0,400}\busize\b',s),'public usize');ck(not re.search(r'use\s+std::(?:fs|process|env|net)\s+as',s),'aliased io');
   if p.name!='mod.rs'and any(x in p.parts for x in('codec','integrity','joc')):
    code='\n'.join(x for x in s.splitlines()if x.strip()and not x.lstrip().startswith('//'));ck(not code,'empty leaf item')
run_checks=lambda root=ROOT,package=PKG:run(root)
def main():
 try:run()
 except(BoundaryError,Exception)as e:print('boundary check failed: '+str(e),file=sys.stderr);return 1
 print('boundary checks passed (2 scoped materializations; 102 inventoried; roots deferred)');return 0
if __name__=='__main__':raise SystemExit(main())
