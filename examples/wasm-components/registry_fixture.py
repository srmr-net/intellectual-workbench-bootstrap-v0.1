"""Construct an installable pure package from the real cargo-component echo build."""
from pathlib import Path
import hashlib
import json
import shutil
ROOT = Path(__file__).resolve().parents[2]
out = ROOT / 'target/registry-fixtures/pure-echo'
if out.exists():
    shutil.rmtree(out)
(out / 'schemas').mkdir(parents=True)
wasm = ROOT / 'target/abi-guests/wasm32-unknown-unknown/release/abi_echo.wasm'
shutil.copyfile(wasm, out / 'logic.wasm')
digest = hashlib.sha256(wasm.read_bytes()).hexdigest()
for name in ['input', 'output']:
    shutil.copyfile(ROOT / 'examples/wasm-components/fixtures/text.schema.json', out / f'schemas/{name}.json')
m = {'manifestVersion': '0.1', 'id': 'org.workbench.registry.echo', 'version': '0.1.0',
     'runtime': {'kind': 'wasm-component', 'interface': 'org.workbench.component@0.1',
                 'module': 'logic.wasm', 'sha256': digest},
     'metadata': 'capabilities.ttl', 'permissions': [],
     'ports': {d: [{'name': n, 'schema': f'schemas/{s}.json',
                    'semanticType': 'https://workbench.example/spike#Text',
                    'unit': 'https://workbench.example/spike#TextUnit'}]
               for d,n,s in [('inputs','input','input'), ('outputs','result','output')]}}
(out / 'component.json').write_text(json.dumps(m, indent=2) + '\n')
(out / 'capabilities.ttl').write_text('''@prefix wb: <https://workbench.example/ns/0.1#> .
@prefix ex: <https://workbench.example/registry/> .
ex:echo a wb:Component ; wb:componentId "org.workbench.registry.echo" ; wb:componentVersion "0.1.0" ;
 wb:providesCapability ex:Echo ; wb:hasInputPort ex:input ; wb:hasOutputPort ex:output ; wb:hasArtifact ex:artifact .
ex:input a wb:InputPort ; wb:portName "input" ; wb:schemaRef "schemas/input.json" ;
 wb:semanticType <https://workbench.example/spike#Text> ; wb:unit <https://workbench.example/spike#TextUnit> .
ex:output a wb:OutputPort ; wb:portName "result" ; wb:schemaRef "schemas/output.json" ;
 wb:semanticType <https://workbench.example/spike#Text> ; wb:unit <https://workbench.example/spike#TextUnit> .
ex:artifact a wb:Artifact ; wb:artifactPath "logic.wasm" ; wb:sha256 "DIGEST" .
'''.replace('DIGEST', digest))
print(f'{digest}  {out.relative_to(ROOT)}/logic.wasm')
