import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 m=json.loads((R/'manifest.json').read_text());s=json.loads((R/'package_seal.json').read_text())
 exp={x['path']:x['sha256'] for x in m['files']};act={p.name:h(p) for p in R.iterdir() if p.is_file() and p.name not in {'manifest.json','package_seal.json'}}
 assert exp==act and s['manifest_sha256']==h(R/'manifest.json')
 e=json.loads((R/'EVIDENCE.json').read_text());src=json.loads((R/'SOURCE-RECEIPTS.json').read_text())
 assert e['status']=='TWO_PATHS_OBSERVED_JOIN_UNBOUND' and e['production_allowed'] is False
 assert len(e['missing_joins'])==5
 assert src['manifest_sha256']=='da17b0f4868858a91cefd8b633f85f578ca00798fbfb8ca55d5e2ef01dff57d9'
 print('PASS sanitized I0/D0 AU boundary; join stays unbound and publication disabled')
if __name__=='__main__':main()
