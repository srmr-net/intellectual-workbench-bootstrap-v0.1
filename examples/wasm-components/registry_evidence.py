"""Verify named REG gates and archive exact inputs after all ABI/IPC checks pass."""
from importlib.metadata import version
from pathlib import Path
import hashlib
import json
import os
import re
import shutil
ROOT = Path(__file__).resolve().parents[2]
REPORT = ROOT / 'target/abi-evidence'
logs = (REPORT / 'workspace-tests.txt').read_text()
ids = sorted(set(re.findall(r'test reg_(\d{3})_\S+ \.\.\. ok', logs)))
assert ids == [f'{n:03d}' for n in range(1,21)]
assert not re.search(r'test reg_\S+ \.\.\. (?:ignored|FAILED)', logs)
run = json.loads((REPORT / 'run.json').read_text())
assert all(v == 'passed' for v in run['checks'].values())
files = {
 'component.schema.json': 'schemas/component.schema.json',
 'ontology.ttl': 'ontology/workbench.ttl',
 'admission-shapes.ttl': 'crates/wb-semantic/admission-shapes.ttl',
 'registry-tests.rs': 'crates/wb-registry/tests/registry.rs',
 'registry.rs': 'crates/wb-registry/src/lib.rs',
 'semantic-worker.py': 'tools/registry_validate.py',
 'inspection-worker.rs': 'crates/wb-registry/src/bin/wb-registry-inspect.rs',
 'worker-limits.rs': 'crates/wb-semantic/src/lib.rs',
 'kernel-adapter.rs': 'crates/wb-kernel/src/runtime.rs',
 'ipc-tests.rs': 'crates/wb-testkit/tests/ipc.rs',
 'requirements.txt': 'tools/requirements.txt',
}
for name, source in files.items():
    shutil.copyfile(ROOT / source, REPORT / 'registry' / name)
shutil.copytree(ROOT / 'target/registry-fixtures/pure-echo', REPORT / 'registry/fixture', dirs_exist_ok=True)
receipt = json.loads((REPORT / 'registry/receipt.json').read_text())
assert hashlib.sha256((REPORT / 'registry/record.json').read_bytes()).hexdigest() == receipt['record_digest']
assert hashlib.sha256((REPORT / 'registry/fixture/logic.wasm').read_bytes()).hexdigest() == receipt['pin']['sha256']
record = json.loads((REPORT / 'registry/record.json').read_text())
assert record['pin'] == receipt['pin'] and record['checked']['trust'] == 'unreviewed'
for name, digest in record['files'].items():
    assert hashlib.sha256((REPORT / 'registry/fixture' / name).read_bytes()).hexdigest() == digest
summary = {
 'headSha': run['headSha'], 'trackedChanges': run['trackedChanges'],
 'cases': {f'REG-{n}': 'production-path tests passed' for n in ids},
 'pyshacl': version('pyshacl'), 'hostedExecution': os.environ.get('GITHUB_ACTIONS') == 'true',
 'policy': {'filesAndDirectories':64, 'fileBytes':2097152, 'packageBytes':8388608,
            'depth':8, 'storeLogicalBytes':134217728, 'releases':128,
            'workerAddressBytes':2147483648, 'workerSeconds':30, 'workerOutputBytes':4194304},
 'crashTests': 'real SIGKILL at seven production checkpoints; reopen/resolve/read/reinstall',
 'limitations': ['Linux local filesystem; fsync durability contract, no physical power-cut test',
                 'private host-owned store and trusted worker binaries; same-UID attacker excluded',
                 'UI-bearing packages and nonempty permissions explicitly blocked',
                 'no removal, remote registry, signatures, dependency solving or trust elevation',
                 'schema references acyclic local files/object-member JSON pointers; IDs/anchors/dynamic refs/array pointers rejected'],
 'acceptance': 'Requires independent review and external verification of hosted success/artifacts',
}
(REPORT / 'registry/report.json').write_text(json.dumps(summary, indent=2)+'\n')
for path in sorted((REPORT / 'registry').rglob('*')):
    if path.is_file():
        run['sha256'][str(path.relative_to(REPORT))] = hashlib.sha256(path.read_bytes()).hexdigest()
run['registryReport'] = 'registry/report.json'
run['pyshacl'] = version('pyshacl')
(REPORT / 'run.json').write_text(json.dumps(run, indent=2)+'\n')
print('PASS: REG-001..020, unchanged ABI/IPC suites and registry evidence archived')
