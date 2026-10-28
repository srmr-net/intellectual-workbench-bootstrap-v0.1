from pathlib import Path
import hashlib
ROOT=Path(__file__).resolve().parents[3]
BASE='57b76ef9835819b46bcc6b2e778df781b1d1c425'
def selected(name):
 return (name.startswith(('apps/desktop-qt/vslice-004/','apps/desktop-qt/vslice-004-tests/')) or name in {'apps/desktop-qt/validate-vslice-004.sh','.github/workflows/vslice-004.yml','docs/implementation/vslice-004/evidence/verify.py','docs/implementation/vslice-004/CONFORMANCE.md'}) and not {'target','__pycache__'}.intersection(Path(name).parts)
def sources():
 files={str(p.relative_to(ROOT)) for directory in ['apps/desktop-qt/vslice-004','apps/desktop-qt/vslice-004-tests'] for p in (ROOT/directory).rglob('*') if p.is_file() and selected(str(p.relative_to(ROOT)))}
 files|={'apps/desktop-qt/validate-vslice-004.sh','.github/workflows/vslice-004.yml','docs/implementation/vslice-004/evidence/verify.py','docs/implementation/vslice-004/CONFORMANCE.md'}
 return sorted(files)

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def assert_private_artifact(data):
 # Synthetic private credential marker, assembled so source archives are clean.
 marker='-'.join(['vs4','private','test','token','not','real']).encode()
 assert marker not in data,'private credential leaked into evidence'
