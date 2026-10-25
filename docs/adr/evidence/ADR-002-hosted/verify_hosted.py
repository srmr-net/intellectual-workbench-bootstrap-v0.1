"""Verify the ADR-002 hosted acceptance artifact against GitHub and checked-in inputs."""
from pathlib import Path
import hashlib
import json
import re
import zipfile

ROOT = Path.cwd()
BASE = ROOT / 'target/adr-002-hosted'
RUN_ID = 40225967487
SHA = 'e19edb2165218de66f64a228fefc75c840ff9cc0'
run = json.loads((BASE / 'run-api.json').read_text())
jobs = json.loads((BASE / 'jobs-api.json').read_text())['jobs']
artifacts = json.loads((BASE / 'artifacts-api.json').read_text())['artifacts']
artifact = next(a for a in artifacts if a['name'] == 'ipc-validation-linux' and not a['expired'])
assert run['id'] == RUN_ID and run['head_sha'] == SHA
assert run['status'] == 'completed' and run['conclusion'] == 'success'
assert {j['name'] for j in jobs} == {'rust', 'contracts', 'ipc-validation (includes ABI)'}
assert all(j['conclusion'] == 'success' for j in jobs)
ipc_job = next(j for j in jobs if j['name'] == 'ipc-validation (includes ABI)')
assert ipc_job['runner_group_name'] == 'GitHub Actions' and 'ubuntu-24.04' in ipc_job['labels']
archive = BASE / 'artifact.zip'
archive_hash = hashlib.sha256(archive.read_bytes()).hexdigest()
assert artifact['digest'] == 'sha256:' + archive_hash
with zipfile.ZipFile(archive) as zipped:
    for name in zipped.namelist():
        assert not Path(name).is_absolute() and '..' not in Path(name).parts
    zipped.extractall(BASE / 'artifact')
r = BASE / 'artifact'
metadata = json.loads((r / 'run.json').read_text())
report = json.loads((r / 'ipc-report.json').read_text())
assert metadata['headSha'] == report['headSha'] == SHA
assert not metadata['trackedChanges']
assert metadata['github']['GITHUB_SHA'] == SHA
assert metadata['github']['GITHUB_RUN_ID'] == str(RUN_ID)
assert metadata['github']['GITHUB_RUN_ATTEMPT'] == str(run['run_attempt'])
assert report['hostedExecution'] and report['witUnchanged']
assert metadata['wasmtime'] == '49.0.1'
assert metadata['rustc'].startswith('rustc 1.99.1 ')
assert metadata['cargoComponent'].endswith(' 0.21.1')
assert all(s == 'passed' for s in metadata['checks'].values())
assert set(report['cases']) == {f'IPC-{n:03d}' for n in range(1,16)}
assert report['abiCases'] == [f'ABI-{n:03d}' for n in range(1,11)]
for name, expected in metadata['sha256'].items():
    assert hashlib.sha256((r / name).read_bytes()).hexdigest() == expected, name
for name, local in [('component.wit','wit/component.wit'), ('native-Cargo.lock','Cargo.lock'), ('guest-Cargo.lock','examples/wasm-components/Cargo.lock'), ('abi-tests.rs','crates/wb-wasm-host/tests/abi.rs'), ('host-adapter.rs','crates/wb-wasm-host/src/lib.rs')]:
    assert (r / name).read_bytes() == (ROOT / local).read_bytes(), name
assert metadata['sha256']['component.wit'] == 'cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0'
assert metadata['sha256']['abi-tests.rs'] == '263f74a47a8daf26197f8b3e947fc51e73e05da4030ac008ea416ed4d9542c95'
logs = re.sub(r'\x1b(?:\[[0-?]*[ -/]*[@-~]|[()][0-~])','',(r / 'workspace-tests.txt').read_text())
for prefix, count in [('abi',10),('ipc',15)]:
    for n in range(1,count+1):
        assert re.search(rf'test {prefix}_{n:03d}_\S+ \.\.\. ok',logs), (prefix,n)
passed = sum(map(int,re.findall(r'test result: ok\. (\d+) passed;',logs)))
assert passed == 47, passed
assert len(re.findall(r'test ipc_\d{3}_\S+ \.\.\. ok',logs)) == 24
assert '30 fixtures' in (r/'contracts.txt').read_text()
assert 'Ran 4 tests' in (r/'python-tests.txt').read_text()
verification = {'runUrl':run['html_url'],'headSha':SHA,'allHostedJobsPassed':True,
    'ipcCasesPassed':15,'ipcTestsPassed':24,'originalAbiCasesPassed':10,'rustTestsPassed':passed,
    'schemaFixturesPassed':30,'pythonHarnessTestsPassed':4,'githubHostedLinux':True,
    'artifactId':artifact['id'],'archiveSha256':archive_hash,'artifactFilesRehashed':len(metadata['sha256']),
    'acceptedWitAbiTestsHostAndLockfilesMatchCheckout':True}
(BASE/'verification.json').write_text(json.dumps(verification,indent=2)+'\n')
print(json.dumps(verification,indent=2))
