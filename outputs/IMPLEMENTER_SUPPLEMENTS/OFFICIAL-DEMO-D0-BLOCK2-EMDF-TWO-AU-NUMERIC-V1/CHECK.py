from __future__ import annotations
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parent
def sha256(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 m=json.loads((ROOT/'manifest.json').read_text(encoding='utf-8'))
 s=json.loads((ROOT/'package_seal.json').read_text(encoding='utf-8'))
 expected={x['path']:x['sha256'] for x in m['files']}
 actual={p.name:sha256(p) for p in ROOT.iterdir() if p.is_file() and p.name not in {'manifest.json','package_seal.json'}}
 assert actual==expected
 assert s['manifest_sha256']==sha256(ROOT/'manifest.json')
 d=json.loads((ROOT/'SKIPFLD-VALUES.json').read_text(encoding='utf-8'))
 assert d['block_index']=={'value':2,'base':'zero-based','audio_block_prefix_ordinal':3}
 assert len(d['samples'])==2
 assert [x['C_bytes'] for x in d['samples']]==[314,301]
 assert all(x['skipfld_present'] and x['C_bytes']==x['emdf_container_consumed_bytes'] and x['trailing_bytes']==0 for x in d['samples'])
 assert all(x['C_bytes']<=d['target_contract']['max_C_bytes'] for x in d['samples'])
 assert [x['payloads'] for x in d['samples']]==[[{'id':11,'data_bytes':79},{'id':14,'data_bytes':216}],[{'id':11,'data_bytes':66},{'id':14,'data_bytes':216}]]
 assert d['target_contract']['overflow']=='REJECT' and d['target_contract']['continuation_across_skipfields']=='DISALLOWED'
 assert d['target_contract']['production_allowed'] is False
 print('PASS official demo two-AU block-2 skipfld numeric vectors; 640-kbps sample scope preserved')
if __name__=='__main__':main()
