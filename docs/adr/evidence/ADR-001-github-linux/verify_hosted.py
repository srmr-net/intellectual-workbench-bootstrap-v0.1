import hashlib
import json
from pathlib import Path
import re
import zipfile

root = Path.cwd()
base = root / 'target/abi-hosted'
run = json.loads((base / 'run-api.json').read_text())
jobs = json.loads((base / 'jobs-api.json').read_text())['jobs']
artifact = json.loads((base / 'artifacts-api.json').read_text())['artifacts']
artifact = next(a for a in artifact if a['name'] == 'abi-validation-linux')
expected_sha = 'ed73ec7238f1e21192fcbf0bb5698ed1b6197986'
assert run['id'] == 38807099576 and run['head_sha'] == expected_sha
assert run['status'] == 'completed' and run['conclusion'] == 'success'
assert {j['name'] for j in jobs} == {'rust', 'contracts', 'abi-validation'}
assert all(j['conclusion'] == 'success' for j in jobs)
abi = next(j for j in jobs if j['name'] == 'abi-validation')
assert abi['runner_group_name'] == 'GitHub Actions' and 'ubuntu-24.04' in abi['labels']
archive = base / 'artifact.zip'
zip_hash = hashlib.sha256(archive.read_bytes()).hexdigest()
assert artifact['digest'] == f'sha256:{zip_hash}'
with zipfile.ZipFile(archive) as z:
    for member in z.namelist():
        assert not Path(member).is_absolute() and '..' not in Path(member).parts
    z.extractall(base / 'artifact')
directory = base / 'artifact'
metadata = json.loads((directory / 'run.json').read_text())
assert metadata['headSha'] == expected_sha and not metadata['trackedChanges']
assert metadata['github']['GITHUB_SHA'] == expected_sha
assert metadata['github']['GITHUB_RUN_ID'] == str(run['id'])
assert metadata['github']['RUNNER_OS'] == 'Linux'
assert metadata['wasmtime'] == '49.0.1'
assert metadata['rustc'].startswith('rustc 1.99.1 ')
assert metadata['cargoComponent'].endswith(' 0.21.1')
assert all(s == 'passed' for s in metadata['checks'].values())
for name, expected in metadata['sha256'].items():
    assert hashlib.sha256((directory / name).read_bytes()).hexdigest() == expected
for archived, local in [('native-Cargo.lock', 'Cargo.lock'), ('guest-Cargo.lock', 'examples/wasm-components/Cargo.lock'), ('component.wit', 'wit/component.wit')]:
    assert (directory / archived).read_bytes() == (root / local).read_bytes()
ansi = re.compile(r'\x1b(?:\[[0-?]*[ -/]*[@-~]|[()][0-~])')
logs = ansi.sub('', (directory / 'workspace-tests.txt').read_text())
for number in range(1, 11):
    assert re.search(rf'test abi_{number:03d}_\S+ \.\.\. ok', logs), number
passed = sum(int(n) for n in re.findall(r'test result: ok\. (\d+) passed;', logs))
assert passed == 23, passed
assert '30 fixtures' in (directory / 'contracts.txt').read_text()
assert 'Ran 4 tests' in (directory / 'python-tests.txt').read_text()
verification = {
    'runUrl': run['html_url'], 'headSha': expected_sha,
    'allHostedJobsPassed': True, 'githubHostedLinux': True,
    'allTenAbiTestsPassed': True, 'rustTestsPassed': passed,
    'pythonHarnessTestsPassed': 4, 'schemaFixturesPassed': 30,
    'archiveSha256': zip_hash, 'artifactId': artifact['id'],
    'artifactFilesRehashed': len(metadata['sha256']),
    'witAndLockfilesMatchCheckout': True,
}
(base / 'verification.json').write_text(json.dumps(verification, indent=2) + '\n')
print(json.dumps(verification, indent=2))
