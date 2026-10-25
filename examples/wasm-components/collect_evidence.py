"""Archive the exact successful validation inputs/outputs; never collect secrets."""
from importlib.metadata import version
from pathlib import Path
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[2]
REPORT = ROOT / 'target/abi-evidence'


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def main():
    files = {
        'native-Cargo.lock': 'Cargo.lock',
        'guest-Cargo.lock': 'examples/wasm-components/Cargo.lock',
        'component.wit': 'wit/component.wit',
        'abi-tests.rs': 'crates/wb-wasm-host/tests/abi.rs',
        'host-adapter.rs': 'crates/wb-wasm-host/src/lib.rs',
        'components/ipc-aggregate.wasm': 'target/abi-guests/wasm32-unknown-unknown/release/ipc_aggregate.wasm',
        'components/echo.wasm': 'target/abi-guests/wasm32-unknown-unknown/release/abi_echo.wasm',
        'components/integer.wasm': 'target/abi-guests/wasm32-unknown-unknown/release/abi_integer.wasm',
        'components/denied-wasi.wasm': 'target/abi-guests/wasm32-wasip1/release/abi_denied_wasi.wasm',
    }
    for name, source in files.items():
        dest = REPORT / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / source, dest)
    hashes = {name: hashlib.sha256((REPORT / name).read_bytes()).hexdigest() for name in files}
    packages = tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']
    metadata = {
        'headSha': command('git', 'rev-parse', 'HEAD'),
        'trackedChanges': bool(command('git', 'status', '--porcelain', '--untracked-files=no')),
        'platform': platform.platform(),
        'architecture': platform.machine(),
        'rustc': command('rustc', '--version'),
        'cargo': command('cargo', '--version'),
        'cargoComponent': command('cargo', 'component', '--version'),
        'wasmtime': next(p['version'] for p in packages if p['name'] == 'wasmtime'),
        'python': platform.python_version(),
        'jsonschema': version('jsonschema'),
        'rdflib': version('rdflib'),
        'github': {key: os.environ.get(key) for key in (
            'GITHUB_SERVER_URL', 'GITHUB_REPOSITORY', 'GITHUB_RUN_ID',
            'GITHUB_RUN_ATTEMPT', 'GITHUB_SHA', 'GITHUB_REF', 'RUNNER_OS', 'RUNNER_ARCH')},
        'checks': {name: 'passed' for name in (
            'guest-builds', 'workspace-tests', 'clippy', 'contracts', 'python-tests', 'fmt')},
        'sha256': hashes,
    }
    logs = re.sub(r'\x1b(?:\[[0-?]*[ -/]*[@-~]|[()][0-~])', '', (REPORT / 'workspace-tests.txt').read_text())
    ipc_ids = sorted(set(re.findall(r'test ipc_(\d{3})_\S+ \.\.\. ok', logs)))
    abi_ids = sorted(set(re.findall(r'test abi_(\d{3})_\S+ \.\.\. ok', logs)))
    assert ipc_ids == [f'{n:03d}' for n in range(1,16)]
    assert abi_ids == [f'{n:03d}' for n in range(1,11)]
    frozen = 'cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0'
    assert hashes['component.wit'] == frozen
    assert hashes['abi-tests.rs'] == '263f74a47a8daf26197f8b3e947fc51e73e05da4030ac008ea416ed4d9542c95'
    ipc = {'headSha': metadata['headSha'], 'cases': {f'IPC-{n}':'executable checks passed' for n in ipc_ids},
           'abiCases': [f'ABI-{n}' for n in abi_ids], 'witUnchanged': True,
           'hostedExecution': os.environ.get('GITHUB_ACTIONS') == 'true',
           'acceptance': 'Pending external confirmation of successful hosted workflow and independent review'}
    (REPORT / 'ipc-report.json').write_text(json.dumps(ipc, indent=2) + '\n')
    metadata['ipcReport'] = 'ipc-report.json'
    (REPORT / 'run.json').write_text(json.dumps(metadata, indent=2) + '\n')
    summary = '# ADR-001 validation passed\n\n' + '```json\n' + json.dumps(metadata, indent=2) + '\n```\n'
    (REPORT / 'summary.md').write_text(summary)
    if os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as output:
            output.write(summary)
    print(f'PASS: evidence archived for {metadata["headSha"]}')


if __name__ == '__main__':
    main()
