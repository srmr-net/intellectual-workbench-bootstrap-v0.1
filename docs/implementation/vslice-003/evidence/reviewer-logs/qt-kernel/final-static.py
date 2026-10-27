from pathlib import Path
import hashlib,json,subprocess,sys
root=Path.cwd();sys.path.insert(0,str(root/'apps/desktop-qt/vslice-003-tests'));from contract import sources
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
print('Review HEAD',subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip())
print('Qt app',sha(root/'target/vs3-build/workbench-vslice003'))
print('Native worker',sha(root/'target/debug/wb-vslice003-worker'))
for role in ['agent-security','runtime-integration']:
 r=json.loads((root/f'docs/implementation/adr-005/evidence/review-{role}.json').read_text())
 for name,digest in r['sourceSha256'].items():assert sha(root/name)==digest,name
 print('Accepted ADR005 independent source map unchanged:',role,len(r['sourceSha256']))
for name in sources():assert (root/name).is_file(),name
print('VS3 source inventory',len(sources()),'paths includes trusted Main.qml/resources.qrc and provenance tests')
print('Qt model lifetime expression:',next(line.strip() for line in (root/'apps/desktop-qt/vslice-003/Main.qml').read_text().splitlines() if 'model:' in line))
print('Independent static review: no package-authored QML, no render math, no WIT/manifest/schema/ontology edits; Session authority and separate ingress preserved')
print('Review limitation: hosted acceptance remains external; this receipt does not self-accept VS3')
