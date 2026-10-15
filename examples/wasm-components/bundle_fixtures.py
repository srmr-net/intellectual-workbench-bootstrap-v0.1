"""Write non-installable ABI-only manifest fixtures, hashing the built raw bytes."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[2]
for name, target in [('echo', 'wasm32-unknown-unknown'), ('integer', 'wasm32-unknown-unknown'),
                     ('denied-wasi', 'wasm32-wasip1')]:
    wasm = ROOT / f'target/abi-guests/{target}/release/abi_{name.replace("-", "_")}.wasm'
    digest = hashlib.sha256(wasm.read_bytes()).hexdigest()
    manifest = {
        'manifestVersion': '0.1', 'id': f'org.workbench.spike.{name}', 'version': '0.1.0',
        'runtime': {'kind': 'wasm-component', 'interface': 'org.workbench.component@0.1',
                    'module': 'logic.wasm', 'sha256': digest},
        'metadata': 'capabilities.ttl', 'permissions': [],
        'ports': {direction: [{'name': port, 'schema': 'payload.schema.json',
                              'semanticType': 'https://workbench.example/spike#Payload'}]
                  for direction, port in [('inputs', 'input'), ('outputs', 'result')]}}
    output = ROOT / f'target/abi-fixtures/{name}'
    output.mkdir(parents=True, exist_ok=True)
    (output / 'component.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'{digest}  {wasm.relative_to(ROOT)}')
