import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 m=json.loads((R/'manifest.json').read_text());s=json.loads((R/'package_seal.json').read_text())
 exp={x['path']:x['sha256'] for x in m['files']};act={p.name:h(p) for p in R.iterdir() if p.is_file() and p.name not in {'manifest.json','package_seal.json'}}
 assert exp==act and s['manifest_sha256']==h(R/'manifest.json')
 x=json.loads((R/'SOURCE-RECEIPTS.json').read_text())['observed']
 assert x['direct_worker_range_reads']==6 and x['direct_worker_range_stores']==0
 assert x['external_populator']=='UNBOUND' and x['global_absence_claim'] is False and x['production_allowed'] is False
 print('PASS bounded ProEncoder source-range method-set receipt; owner stays unbound')
if __name__=='__main__':main()
