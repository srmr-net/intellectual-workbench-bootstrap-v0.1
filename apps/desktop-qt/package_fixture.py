"""New immutable UI identity, exact accepted mortgage WASM; no headless rewrite."""
from pathlib import Path
import json,shutil,hashlib
ROOT=Path(__file__).resolve().parents[2]
source=ROOT/'target/mortgage-packages/mortgage-headless'
out=ROOT/'target/ui-fixtures/mortgage-ui'
if out.exists():shutil.rmtree(out)
shutil.copytree(source,out)
m=json.loads((out/'component.json').read_text());old=m['id'];m['id']='org.workbench.example.mortgage-ui';m['ui']='ui.json'
(out/'component.json').write_text(json.dumps(m,indent=2)+'\n')
shutil.copyfile(ROOT/'examples/mortgage-ui.json',out/'ui.json')
p=out/'capabilities.ttl';s=p.read_text().replace(old,m['id']).replace('wb:hasArtifact ex:artifact .','wb:hasArtifact ex:artifact ; wb:hasInterface ex:form .')
s+='\nex:form a wb:AbstractInterface ; wb:uiDefinitionRef "ui.json" .\n';p.write_text(s)
assert hashlib.sha256((out/'logic.wasm').read_bytes()).hexdigest()==m['runtime']['sha256']
print('UI fixture:',out.relative_to(ROOT),'WASM:',m['runtime']['sha256'],'UI:',hashlib.sha256((out/'ui.json').read_bytes()).hexdigest())
# Independent metadata shape over the same guest proves the renderer is generic.
alternate=out.parent/'alternate-ui'
if alternate.exists():shutil.rmtree(alternate)
shutil.copytree(out,alternate)
m['id']='org.workbench.example.alternate-ui'
(alternate/'component.json').write_text(json.dumps(m,indent=2)+'\n')
p=alternate/'capabilities.ttl';p.write_text(p.read_text().replace('org.workbench.example.mortgage-ui',m['id']))
u=json.loads((alternate/'ui.json').read_text());u['root']['title']='Alternate semantic form'
for c,name in zip(u['root']['children'],['amount','interest','duration','result']):c['id']=name
u['root']['children'][0]['label']='Long accessible amount label ' * 5
(alternate/'ui.json').write_text(json.dumps(u,indent=2)+'\n')
malicious=out.parent/'malicious-ui'
if malicious.exists():shutil.rmtree(malicious)
shutil.copytree(out,malicious)
u=json.loads((malicious/'ui.json').read_text());u['root']['children'][0]['onChanged']='Qt.quit()'
(malicious/'ui.json').write_text(json.dumps(u)+'\n')
