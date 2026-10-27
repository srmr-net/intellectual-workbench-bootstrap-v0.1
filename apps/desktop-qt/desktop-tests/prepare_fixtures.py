"""TEST-only UI identity over the already compiled accepted adversarial guest."""
from pathlib import Path
import hashlib,json,shutil
ROOT=Path(__file__).resolve().parents[3]
source=ROOT/'target/ui-fixtures/mortgage-ui'
out=ROOT/'target/vs2-fixtures/adversary-ui'
if out.exists():shutil.rmtree(out)
shutil.copytree(source,out)
m=json.loads((out/'component.json').read_text());old_id=m['id'];old_sha=m['runtime']['sha256']
wasm=(ROOT/'target/mortgage-packages/mortgage-adversary/logic.wasm').read_bytes();digest=hashlib.sha256(wasm).hexdigest()
m['id']='org.workbench.example.vs2-adversary-ui';m['runtime']['sha256']=digest
(out/'component.json').write_text(json.dumps(m,indent=2)+'\n');(out/'logic.wasm').write_bytes(wasm)
p=out/'capabilities.ttl';p.write_text(p.read_text().replace(old_id,m['id']).replace(old_sha,digest))
print('TEST adversary UI:',m['id'],digest)
