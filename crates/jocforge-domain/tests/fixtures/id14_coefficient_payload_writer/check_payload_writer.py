"""Input-first independent bit construction. No DLL or native output is an encoder input."""
import hashlib
import json
import struct
from pathlib import Path

ROOT=Path(__file__).resolve().parent
BANDS=[1,3,5,7,9,12,15,23]
def require(condition,message):
    if not condition:raise RuntimeError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def r32(v):return struct.unpack('<f',struct.pack('<f',v))[0]
def quantize(v,mode):
    fine=mode in (1,3);center=96 if fine else 48;bias=102 if fine else 205;divisor=205 if fine else 410
    if v==0:return center
    scaled=r32(r32(v)*32768)
    require(-2147483648<=scaled<2147483648,'unsupported input')
    integer=max(-32768,min(32767,round(scaled)))
    biased=integer+(-bias if integer<0 else bias)
    quotient=(abs(biased)//divisor)*(-1 if biased<0 else 1)
    return max(0,min(center*2-1,quotient+center))
def bits(value,width):return format(value,'0'+str(width)+'b') if width else ''
def packed(bitstring):return int(bitstring+'0'*((-len(bitstring))%8),2).to_bytes((len(bitstring)+7)//8,'big') if bitstring else b''
def encode(case,books):
    head=bits(case['downmix'],3)+bits(len(case['objects'])-1,6)+bits(case['extension'],3)+bits(case['clip'],8)+bits(case['sequence'],10)
    body='';quantized=[]
    def code(name,symbol):
        entry=books[name]['entries'][symbol]
        require(entry['symbol']==symbol,'codebook symbol order')
        return bits(entry['code'],entry['bit_length'])
    for i,obj in enumerate(case['objects']):
        head+=bits(int(obj['present']),1)
        if not obj['present']:continue
        head+=bits(obj['band_enum'],3)+bits(obj['mode'],2)+bits(int(obj['offsets'] is not None),1)+bits(len(obj['points'])-1,1)
        if obj['offsets'] is not None:
            for offset in obj['offsets']:head+=bits(offset,5)
        mode=obj['mode'];steps=192 if mode in (1,3) else 96;count=BANDS[obj['band_enum']]
        for p,point in enumerate(obj['points']):
            if mode in (2,3):
                selected=obj['selected'][p];channels=len(point)
                body+=bits(selected[0],3)
                for b in range(1,count):body+=code('position_'+str(channels),(selected[b]-selected[b-1])%channels)
                previous=100 if mode==3 else 50;values=[]
                for b in range(count):
                    q=quantize(point[selected[b]][b],mode);values.append(q)
                    body+=code('sparse_fine' if mode==3 else 'sparse_coarse',(q-previous)%steps);previous=q
            else:
                values=[]
                for row in point:
                    previous=steps//2;v=[]
                    for coefficient in row[:count]:
                        q=quantize(coefficient,mode);v.append(q)
                        body+=code('full_fine' if mode==1 else 'full_coarse',(q-previous)%steps);previous=q
                    values.append(v)
            quantized.append(dict(object=i,point=p,values=values))
    return head,body,quantized

def main():
    identities=json.loads((ROOT/'SOURCE-IDENTITIES.json').read_text())
    for filename,digest in identities['local_copies'].items():require(sha((ROOT/filename).read_bytes())==digest,'source copy hash '+filename)
    books=json.loads((ROOT/'approved-codebooks.json').read_text())['tables']
    native_books=json.loads((ROOT/'NATIVE-CODEBOOK-POSTCOMPARE.json').read_text())
    require(native_books['dll_sha256']==identities['dll']['sha256'],'native table DLL')
    total_symbols=0
    for row in native_books['rows']:
        expected=[{k:e[k] for k in ('symbol','code','bit_length')} for e in books[row['table']]['entries']]
        require(row['entries']==expected,'full native codebook '+row['table'])
        total_symbols+=len(expected)
    require(total_symbols==588,'six codebook coverage')
    inputs=json.loads((ROOT/'INPUT-ONLY.json').read_text());native=json.loads((ROOT/'NATIVE-OBSERVATIONS.json').read_text())
    require(native['dll_sha256']==identities['dll']['sha256'],'DLL binding')
    require(len(inputs)==len(native['cases'])==15,'case inventory')
    rows=[]
    for case,observed in zip(inputs,native['cases']):
        name=case['name'];require(name==observed['name'],'case join')
        head,body,q=encode(case,books)
        require(q==observed['quantized'],'quantized '+name)
        require(observed['guard_intact'],'output guard '+name)
        expected_body_return=2 if len(body)>4344 else 0
        require(observed['body_return']==expected_body_return,'body capacity '+name)
        if expected_body_return:
            require(observed['writer_return'] is None,'failed builder must skip writer')
            rows.append(dict(name=name,result='PASS',boundary='COEFFICIENT_SCRATCH_OVERFLOW'));continue
        require(observed['body_bits']==len(body),'body bits '+name)
        require(observed['body_hex']==packed(body).hex(),'body bits comparison '+name)
        invalid_wire=any(obj['offsets'] and any(v>=32 for v in obj['offsets']) for obj in case['objects'] if obj['present'])
        expected_return=3 if invalid_wire else 2 if (len(head)+len(body)+7)//8>case['capacity'] else 0
        require(observed['writer_return']==expected_return,'writer return '+name)
        if expected_return:
            require(observed['length_in_out']==case['capacity'],'failure length '+name)
            require(observed['output_changed_on_failure'],'partial native failure '+name)
            require(observed['descriptor_join'] is None,'failed output must not queue')
        else:
            payload=packed(head+body)
            require(observed['payload_hex']==payload.hex(),'payload '+name)
            require(observed['length_in_out']==len(payload),'length '+name)
            expected_status=not any(obj['offsets'] and (any(v>=case['max_offset'] for v in obj['offsets']) or len(obj['offsets'])==2 and obj['offsets'][0]>=obj['offsets'][1]) for obj in case['objects'] if obj['present'])
            require(observed['writer_status']==int(expected_status),'writer semantic status '+name)
            join=observed['descriptor_join']
            require(join['return_code']==0 and join['descriptor_id']==14 and join['length']==len(payload) and join['occupied']==1,'descriptor identity '+name)
            require(join['source_is_exact_writer_output'] and join['cache_pointer_preserved'] and join['storage_distinct'],'pointer join '+name)
            require(join['body_sha256']==sha(payload),'descriptor body '+name)
        if expected_return==3:
            rows.append(dict(name=name,result='PASS',boundary='OFFSET_WIRE_DOMAIN_REJECTED_AFTER_PREFIX',writer_return=expected_return))
        else:
            rows.append(dict(name=name,result='PASS',header_bits=len(head),coefficient_bits=len(body),effective_bits=len(head)+len(body),writer_return=expected_return))
    result=dict(result='PASS',cases=len(rows),codebook_symbols=total_symbols,rows=rows,scope='SELF_OWNED_NATIVE_QUANTIZER_BODY_WRITER_AND_DESCRIPTOR_COPY; NOT_FULL_PRODUCTION_PARENT')
    print(json.dumps(result,indent=2))
if __name__=='__main__':main()
