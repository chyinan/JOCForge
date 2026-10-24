import json
from pathlib import Path
import sys

def queue_state(source):
    capacity,delay,period=(source[k] for k in ('capacity','delay','period'))
    if period<=0 or delay<=0:raise ValueError('Outside observed cold queue domain')
    count=(delay+period-1)//period
    return {'capacity_bytes':capacity,'write_selector':int(delay%period!=0),'read_index':count-1,'phase':delay%period,'period':period,'owned_descriptor_count':count,'occupied':[0]*count}

def main():
    root=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parent
    inputs=json.loads((root/'INPUT-ONLY.json').read_text(encoding='utf-8'))
    queues=[queue_state(x) for x in inputs['queue_constructor_inputs']]
    gate=[];progress=inputs['gate']['cold_progress'];threshold=inputs['gate']['threshold_samples'];hop=inputs['gate']['hop_samples']
    frame_bytes=inputs['gate']['bitrate_kbps']*1000*hop//(inputs['gate']['sample_rate_hz']*8)
    for _ in range(inputs['gate']['calls']):
        if progress<threshold+hop:progress+=hop
        gate.append({'progress':progress,'caller_visible_bytes':str(frame_bytes if progress>threshold else 0),'return_status':0})
    tolerances=[];tolerance=inputs['parent']['cold_tolerance']
    for status in inputs['parent']['measured_assembly_return_boundary_inputs']:
        failed=bool(status and tolerance==0)
        if status and tolerance:tolerance-=1
        tolerances.append({'assembly_tolerance':tolerance,'failure_marker':int(failed),'return_status':status if failed else 0})
    expected=json.loads((root/'NATIVE-OBSERVATIONS.json').read_text(encoding='utf-8'))['events']
    errors=[]
    natural=[x for x in expected if x['event']=='native_queue_constructed']
    independent=[x for x in expected if x['event']=='independent_queue_cold_case']
    for index,own in enumerate(queues):
        for case in (natural[index]['queue'],independent[index]['queue']):
            for key,value in own.items():
                if case[key]!=value:errors.append(['queue',index,key])
        if not independent[index]['all_queue_graph_storage_owned'] or not independent[index]['guard_unchanged']:errors.append(['ownership',index])
    natural_tokens={x['queue']['owner'] for x in natural}
    if any(x['queue']['owner'] in natural_tokens for x in independent):errors.append('independent queue aliases production')
    observed_gate=[x for x in expected if x['event']=='frontend_gate_leave']
    for own,native in zip(gate,observed_gate):
        for key,value in own.items():
            if native[key]!=value:errors.append(['gate',native['frontend_invocation'],key])
    leaves=[x for x in expected if x['event']=='selected_parent_leave']
    for own,native in zip(tolerances,leaves):
        for key in ('assembly_tolerance','failure_marker'):
            if own[key]!=native['roots'][key]:errors.append(['parent',native['parent_invocation'],key])
        if own['return_status']!=native['return_status']:errors.append(['parent return',native['parent_invocation']])
    cold=next(x for x in expected if x['event']=='production_top_constructor_leave')
    enters=[x for x in expected if x['event']=='selected_parent_enter']
    frontend=next(x for x in expected if x['event']=='frontend_cold_initialized')
    if cold['top_owner']!=frontend['top_owner'] or cold['parents'][0]['parent_owner']!=enters[0]['roots']['parent_owner']:errors.append('cold owner join')
    for start,end,publication in zip(enters,leaves,observed_gate):
        if start['roots']['parent_owner']!=end['roots']['parent_owner'] or start['frontend_invocation']!=end['frontend_invocation'] or start['frontend_invocation']!=publication['frontend_invocation']:errors.append('invocation identity')
    if len(queues)!=len(natural) or len(queues)!=len(independent) or len(gate)!=len(observed_gate) or len(tolerances)!=len(leaves):errors.append('case count')
    print(json.dumps({'scope':'COLD_QUEUE_RULES_GATE_TOLERANCE_AND_OWNER_JOIN_ONLY','queue_constructs_per_mode':len(queues),'frontend_calls':len(gate),'parent_boundary_replays':len(tolerances),'mismatches':errors,'expected_loaded_after_calculation':True,'assembly_statuses_are_measured_boundary_inputs':True,'complete_parent_or_codec_reimplemented':False,'production_cold_parent_equivalence':False,'failure_injection_or_durable_file_commit_replayed':False},indent=2))
    if errors:raise SystemExit(1)

if __name__=='__main__':main()
