import hashlib,json,struct
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def require(ok,message):
    if not ok:raise RuntimeError(message)
def r(v):return struct.unpack('<f',struct.pack('<f',v))[0]
def word(v):return struct.unpack('<I',struct.pack('<f',v))[0]
def value(w):return struct.unpack('<f',struct.pack('<I',w))[0]
def peak_transform(peak,k):
    clipped=min(1.0,max(-1.0,peak));w=word(clipped)
    exponent=(((2*w)&0xffffffff)>>24)-126 if ((2*w)&0xffffffff) else -127
    n=max(0,min(60,-exponent))
    if n>=48 or peak==0:return 1.0
    t=r(peak*2.0**n)
    polynomial=r(r(k['polynomial_a']-r(t*k['polynomial_b']))+r(r(t*t)*k['polynomial_c']))
    return r(r(polynomial*4.0)+r(r(float(n))*k['exponent_scale']))
def model_step(state,rows,k,thresholds,attenuations,wrapper,previous_tail):
    state=dict(state);count=len(rows[0]);non_lfe=[row for i,row in enumerate(rows) if i!=3]
    tail=max([0.0]+[abs(v) for row in non_lfe for v in row[count-256:]])
    peak=max([value(state['peak_word']),tail]+[abs(v) for row in non_lfe for v in row[:count-256]])
    transformed=peak_transform(peak,k)
    difference=r(transformed-value(state['normalization_word']))
    candidate=r(r(max(-1.0,min(1.0,float(difference)*6.0)))-k['level_bias'])
    old=value(state['level_word']);release=r(old+k['release_step'])
    if release>candidate:
        next_level=candidate;state['hold']=5;branch='ATTACK'
    elif state['hold']:
        next_level=old;state['hold']-=1;branch='HOLD'
    else:next_level=release;branch='RELEASE'
    state['peak_word']=word(tail);state['level_word']=word(next_level)
    index=0;comparison=min(next_level,0.0)
    while index<127 and comparison<thresholds[index]:index+=1
    clip=128+index;output_digest=None
    if wrapper:
        attenuation=attenuations[index];old_atten=value(state['previous_attenuation_word'])
        increment=r(r(attenuation-old_atten)*r(1/256));current=old_atten
        delayed=[previous_tail[i]+row[:count-256] for i,row in enumerate(rows)]
        next_tail=[row[-256:] for row in rows]
        for sample in range(256):
            multiplier=r(r(0.0625)/current)
            for row in delayed:row[sample]=r(r(multiplier*row[sample])*2.0**state['gain_exponent'])
            current=r(current+increment)
        multiplier=r(r(0.0625)/attenuation)
        for row in delayed:
            for sample in range(256,count):row[sample]=r(r(multiplier*row[sample])*2.0**state['gain_exponent'])
        output_digest=hashlib.sha256(b''.join(struct.pack('<'+str(count)+'f',*row) for row in delayed)).hexdigest()
        previous_tail=next_tail;state['previous_attenuation_word']=word(attenuation)
        # 2*256 backing ring; write begins0, read256, each call advances256.
        state['delay_read']=256 if state['delay_read']==512 else state['delay_read']+256
        state['delay_write']=256 if state['delay_write']==512 else state['delay_write']+256
    return state,clip,output_digest,previous_tail,dict(peak=peak,tail=tail,branch=branch)
def main():
    identities=json.loads((ROOT/'SOURCE-IDENTITIES.json').read_text())
    require(hashlib.sha256((ROOT/'FUNCTIONAL-PARAMETERS.json').read_bytes()).hexdigest()==identities['functional_parameters_sha256'],'functional provider source binding')
    params=json.loads((ROOT/'FUNCTIONAL-PARAMETERS.json').read_text())
    thresholds=[value(w) for w in params['threshold_words']];attenuations=[value(w) for w in params['attenuation_words']]
    k={name:value(w) for name,w in params['math_constants'].items()}
    require(len(thresholds)==len(attenuations)==128,'provider extent')
    for i,a in enumerate(attenuations):require(word(a)==word((32+i%32)*2.0**(i//32)/512),'attenuation generation')
    cases=json.loads((ROOT/'INPUT-ONLY.json').read_text());native=json.loads((ROOT/'NATIVE-OBSERVATIONS.json').read_text())
    require(native['dll_sha256']=='3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2','DLL')
    require(len(cases)==len(native['cases'])==393,'case inventory')
    generations=0;branches={};indexes=set()
    for case,observed in zip(cases,native['cases']):
        require(case['name']==observed['name'],'case binding')
        initial=dict(peak_word=word(0),hold=0,level_word=word(0),gain_exponent=case['gain_exponent'],normalization_word=word(r(float(case['gain_exponent']))/48.0),previous_attenuation_word=word(0.0625),delay_read=256,delay_write=0)
        require(initial==observed['constructor'],'constructor '+case['name'])
        state=dict(initial);previous_tail=[[0.0]*256 for _ in range(6)]
        if case['initial']:state.update(case['initial'])
        require(len(case['frames'])==len(observed['generations']),'generation count')
        for g,(recipe,event) in enumerate(zip(case['frames'],observed['generations'])):
            require(event['before']==state,'before '+case['name']+':'+str(g))
            rows=[[0.0]*1536 for _ in range(6)]
            for row,index,w in recipe['spikes']:rows[row][index]=value(w)
            state,clip,digest,previous_tail,diagnostic=model_step(state,rows,k,thresholds,attenuations,case['kind']=='wrapper',previous_tail)
            prefix=case['name']+':'+str(g)
            require(state==event['after'],'after '+prefix+' model='+str(state)+' native='+str(event['after']))
            require(clip==event['clip'],'clip '+prefix)
            require(digest==event['synthetic_compensated_rows_sha256'],'synthetic compensation '+prefix)
            require(event['generation']==g and event['same_owner_as_prior_generation'] and event['owner_token']==case['name']+':owner','owner generation '+prefix)
            require(event['clip_output_pointer_is_writer_context_plus16'],'field pointer '+prefix)
            require(event['writer_return']==0 and event['writer_validity']==1,'writer '+prefix)
            payload=bytes.fromhex(event['payload_hex']);bitstring=''.join(format(v,'08b') for v in payload)
            require(int(bitstring[12:20],2)==clip and int(bitstring[20:30],2)==g%1024,'same generation wire fields '+prefix)
            expected=format(3,'03b')+format(14,'06b')+'000'+format(clip,'08b')+format(g%1024,'010b')+'0'*15
            expected+='0'*((-len(expected))%8)
            require(bitstring==expected,'complete synthetic header '+prefix)
            generations+=1;branches[diagnostic['branch']]=branches.get(diagnostic['branch'],0)+1;indexes.add(clip)
    require(generations==434,'generation inventory')
    require(indexes==set(range(128,256)),'all native producer indices')
    print(json.dumps(dict(result='PASS',cases=len(cases),generations=generations,branches=branches,all_native_producer_indices_128_through_255=True,full_synthetic_compensation_hashes=True,real_pcm_read_or_hashed=False,scope='SELF_OWNED_CLIP_OWNER_CONTINUITY_AND_SAME_CONTEXT_WRITER; NOT_FULL_PRODUCTION_PARENT'),indent=2))
if __name__=='__main__':main()
