import hashlib
import json
from pathlib import Path
import sys

def ID11_capacity(objects,frames):
    return (123*objects*frames+20*frames+objects+903)//8+1

def main():
    root=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parent
    inputs=json.loads((root/'INPUT-ONLY.json').read_text(encoding='utf-8'))
    seed=bytes.fromhex(inputs['ID11_seed_hex'])
    if hashlib.sha256(seed).hexdigest()!=inputs['ID11_seed_sha256']:raise ValueError('Seed identity')
    actual=[]
    for case in inputs['cases']:
        capacity=ID11_capacity(case['object_capacity'],case['frame_capacity']) if case['payload_id']==11 else case['queue_capacity']
        admitted=case['supplied_bytes']<=capacity
        actual.append({'capacity':capacity,'queue_return':0 if admitted else 5,'occupied':[int(admitted),0,0],'return_status':(0 if admitted else 3) if case['entry']=='GENERAL_GROUP_PUSH' else (0 if admitted else 5) if case['entry']=='GENERIC_ENQUEUE' else None})
    expected=json.loads((root/'NATIVE-OBSERVATIONS.json').read_text(encoding='utf-8'))['cases']
    errors=[];wrapper_consistency=[]
    for case,own,native in zip(inputs['cases'],actual,expected):
        for key,value in (('native_compared_capacity',own['capacity']),('native_compared_payload_bytes',case['supplied_bytes']),('queue_return',own['queue_return']),('queue_occupied_after',own['occupied'])):
            if native[key]!=value:errors.append([case['case_id'],key])
        if own['return_status'] is not None and native['return_status']!=own['return_status']:errors.append([case['case_id'],'wrapper mapping'])
        if not native['guard_unchanged']:errors.append([case['case_id'],'guard'])
        if any(x['declared_body_bytes']!=case['supplied_bytes'] or x.get('stored_body_equals_supplied',True) is not True for x in native['cached']):errors.append([case['case_id'],'cached length or copy'])
        if case['entry']=='ID11_INGRESS':
            # Native child-parser receipts are checked, not used as model seeds.
            okay=(native['parser_return']==0 and native['parser_consumed_bits']+native['parser_remaining_bits']==8*case['supplied_bytes'] and native['return_status']==0 and native['owner_valid_after']==1)
            wrapper_consistency.append(okay)
            if not okay:errors.append([case['case_id'],'native ID11 ingress receipt'])
    if len(actual)!=len(expected):errors.append('case count')
    print(json.dumps({'scope':'QUEUE_BYTE_ADMISSION_AND_GENERAL_GROUP_STATUS_MAPPING','cases':len(actual),'mismatches':errors,'ID11_capacity_from_constructor_formula':ID11_capacity(16,8),'expected_loaded_after_calculation':True,'ID11_semantic_parser_reimplemented':False,'ID11_ingress_native_receipts_consistent':sum(wrapper_consistency),'approved160_policy_implemented_or_changed':False},indent=2))
    if errors:raise SystemExit(1)

if __name__=='__main__':main()
