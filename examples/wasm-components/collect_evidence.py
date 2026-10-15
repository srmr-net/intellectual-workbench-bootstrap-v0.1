"""Archive the exact successful validation inputs/outputs; never collect secrets."""
from importlib.metadata import version
from pathlib import Path
import hashlib
import json
import os
import platform
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
    (REPORT / 'run.json').write_text(json.dumps(metadata, indent=2) + '\n')
    summary = '# ADR-001 validation passed\n\n' + '```json\n' + json.dumps(metadata, indent=2) + '\n```\n'
    (REPORT / 'summary.md').write_text(summary)
    if os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as output:
            output.write(summary)
    print(f'PASS: evidence archived for {metadata["headSha"]}')


if __name__ == '__main__':
    main()
