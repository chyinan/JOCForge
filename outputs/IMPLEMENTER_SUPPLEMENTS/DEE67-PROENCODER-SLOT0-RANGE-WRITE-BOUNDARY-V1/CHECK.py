import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 m=json.loads((R/'manifest.json').read_text());s=json.loads((R/'package_seal.json').read_text())
 exp={x['path']:x['sha256'] for x in m['files']};act={p.name:h(p) for p in R.iterdir() if p.is_file() and p.name not in {'manifest.json','package_seal.json'}}
 assert exp==act and s['manifest_sha256']==h(R/'manifest.json')
 x=json.loads((R/'SOURCE-RECEIPTS.json').read_text())
 assert x['slot0_target']=='0x1402ac530' and x['production_allowed'] is False
 assert all(v==0 for v in x['direct_range_stores'].values())
 print('PASS bounded ProEncoder slot-0 range-write contract; no global absence claim')
if __name__=='__main__':main()
