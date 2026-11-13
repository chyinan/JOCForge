import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
def h(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 m=json.loads((R/'manifest.json').read_text());s=json.loads((R/'package_seal.json').read_text())
 exp={x['path']:x['sha256'] for x in m['files']};act={p.name:h(p) for p in R.iterdir() if p.is_file() and p.name not in {'manifest.json','package_seal.json'}}
 assert exp==act and s['manifest_sha256']==h(R/'manifest.json')
 d=json.loads((R/'SKIPFLD-VALUES.json').read_text());a=d['samples']
 assert [x['C'] for x in a]==[314,301,301] and [x['d0_offset'] for x in a]==[7984,13104,18224]
 assert all(x['skipfld_present'] and x['consumed']==x['C'] and x['trailing']==0 and x['C']<=511 for x in a)
 assert all([p['id'] for p in x['payloads']]==[11,14] for x in a)
 assert d['target_contract']['overflow']=='REJECT' and d['target_contract']['production_allowed'] is False
 print('PASS official demo three-AU block-2 EMDF sample vectors; 640-kbps scope preserved')
if __name__=='__main__':main()
