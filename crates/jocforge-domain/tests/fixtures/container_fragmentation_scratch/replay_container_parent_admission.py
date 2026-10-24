"""Control/length replay for an explicit public-provider scratch fixture."""
import json
from pathlib import Path
import sys

def variable_bits(value,width):
    groups=1;limit=1<<width
    while value>=limit:
        limit=(limit+1)<<width;groups+=1
    return groups*(width+1)

def configuration_bits(raw):
    data=bytes.fromhex(raw)
    word=lambda off:int.from_bytes(data[off:off+4],'little')
    result=4
    for flag,off,width in ((12,16,11),(20,24,11),(28,32,2)):
        if data[flag]:result+=variable_bits(word(off),width)
    if data[36]:result+=8
    flags=word(0);result+=1
    if not flags&1:
        aligned=False
        if not data[12]:
            aligned=bool(flags&8);result+=1+2*aligned
        if data[12] or aligned:result+=7
    return result

def calculate(case):
    record_bits=sum(5+configuration_bits(r['configuration_hex'])+variable_bits(r['body_bytes'],8)+8*r['body_bytes'] for r in case['records'])
    cursor=5+record_bits+5+4
    length=(cursor+40+7)//8
    public=case['provider_fixture']=='CALLER_OWNED_PUBLIC_SINGLE_ZERO_BYTE'
    history=case['history_present']
    fields={'counting_return':0,'counted_full_container_bytes':length,'serializer_return':0 if public else 9,'parent_return':0 if public else 7,'published_output_bytes':length if public else 0,'protection_lookup_calls':1,'protection_lookup_status':0 if public else 9,'authentication_initializer_calls':2 if public else 0,'vendor_default_initializer_calls':0,'output_cursor_before_lookup':cursor,'output_cursor_after_serializer':cursor+40*public,'history_present_after':0,'held_header_present_after':history,'secondary_history_used_after':str(int(public)),'secondary_history_write_index_after':'0','secondary_history_read_index_after':'0','serializer_owner_identity_join':True,'output_guard_unchanged':True}
    roots={'selection_scratch','output','published_length'}
    if history:roots.update(('parent','history_flags'))
    if public:roots.update(('secondary_history','secondary_data'))
    segments=[0]*6;remaining=length;first=2 if length<=2040 else 1 if length<=2551 else 0
    for block in range(first,6):
        count=min(remaining,507 if block==first else 511);segments[block]=count;remaining-=count
    return {'fields':fields,'changed_roots':sorted(roots),'plan':{'requested_bytes':length,'planner_return':int(remaining!=0),'segments':segments,'first_block':first,'counter_after':1},'expected_wire_lengths':[n+(4 if b==first else 0) for b,n in enumerate(segments) if n] if public else []}

def main():
    root=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).resolve().parent
    inputs=json.loads((root/'INPUT-ONLY.json').read_text(encoding='utf-8'))
    actual=[calculate(case) for case in inputs['parent_cases']]
    raw_actual=[{'declared_length':n,'nine_bit_wire_length':n&511,'presence_bit':1,'cursor_bits_after':10+8*n,'raw_return_i16':0,'guard_unchanged':True} for n in inputs['direct_raw_reservation_lengths']]
    expected=json.loads((root/'NATIVE-OBSERVATIONS.json').read_text(encoding='utf-8'))
    errors=[]
    for own,native in zip(actual,expected['parent_cases']):
        for field,value in own['fields'].items():
            if native[field]!=value:errors.append([native['invocation'],field,value,native[field]])
        changes=sorted(x['root'] for x in native['changed_owned_root_bytes'] if x['bytes_changed'])
        if changes!=own['changed_roots']:errors.append([native['invocation'],'changed_roots'])
    for own,plan in zip(actual,expected['transport_plans']):
        for field,value in own['plan'].items():
            if plan[field]!=value:errors.append([plan['parent_invocation'],field])
        if [x['wire_length'] for x in plan['fragments']]!=own['expected_wire_lengths']:errors.append([plan['parent_invocation'],'wire_lengths'])
        if any(not x['fragment_equal_to_native_container'] or not x['guard_unchanged'] or x['nine_bit_wire_length']!=x['wire_length'] for x in plan['fragments']):errors.append([plan['parent_invocation'],'native_fragment_receipt'])
    for own,native in zip(raw_actual,expected['raw_reservation_cases']):
        for field,value in own.items():
            if native[field]!=value:errors.append(['raw',own['declared_length'],field])
    if len(actual)!=len(expected['parent_cases']) or len(actual)!=len(expected['transport_plans']):errors.append('count')
    print(json.dumps({'scope':'CONTROL_LENGTH_AND_SCOPED_STATE_EFFECTS_NOT_CRYPTOGRAPHIC_OUTPUT_REPRODUCTION','parent_cases':len(actual),'raw_reservation_cases':len(raw_actual),'mismatches':errors,'expected_loaded_after_computation':True,'authentication_values_computed_or_loaded_by_checker':False,'native_fragment_equality_receipts_checked':True,'fragment_content_independently_reproduced':False,'outer_codec_or_durable_publication_replayed':False},indent=2))
    if errors:raise SystemExit(1)

if __name__=='__main__':main()
