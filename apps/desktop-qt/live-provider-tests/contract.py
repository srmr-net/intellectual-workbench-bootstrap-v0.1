from pathlib import Path
import hashlib
ROOT=Path(__file__).resolve().parents[3]
BASE='08c1e8d9f27db801628ce9cbee42c4c11d35fc3b'
def sources():
 files=[p for directory in ['apps/desktop-qt/live-provider','apps/desktop-qt/live-provider-tests'] for p in (ROOT/directory).rglob('*') if p.is_file() and '__pycache__' not in p.parts and 'target' not in p.parts]
 files += [ROOT/'apps/desktop-qt/validate-live-provider.sh',ROOT/'.github/workflows/adr-006.yml',ROOT/'docs/implementation/adr-006/evidence/verify.py',ROOT/'docs/implementation/adr-006/API_VERIFICATION.md',ROOT/'docs/implementation/adr-006/CONFORMANCE.md']
 files += [p for d in ['schemas','fixtures'] for p in (ROOT/'docs/implementation/adr-006'/d).rglob('*') if p.is_file()]
 return sorted(str(p.relative_to(ROOT)) for p in files)
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
