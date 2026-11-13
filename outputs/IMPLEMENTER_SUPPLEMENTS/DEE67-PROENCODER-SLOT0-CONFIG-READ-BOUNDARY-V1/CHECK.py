import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 m=json.loads((R/'manifest.json').read_text());s=json.loads((R/'package_seal.json').read_text())
 exp={x['path']:x['sha256'] for x in m['files']};act={p.name:h(p) for p in R.iterdir() if p.is_file() and p.name not in {'manifest.json','package_seal.json'}}
 assert exp==act and s['manifest_sha256']==h(R/'manifest.json')
 x=json.loads((R/'SOURCE-RECEIPTS.json').read_text())['result']
 assert x['same_object_config_plus_0x58_loads']==0 and x['unresolved_base_plus_0x58_load_candidates']==0
 assert x['scope']=='slot0 only' and x['profile_index_1_join']=='UNBOUND' and x['global_absence_claim'] is False and x['production_allowed'] is False
 print('PASS slot-0-only Config+0x58 read boundary; profile join stays open')
if __name__=='__main__':main()
