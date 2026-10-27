"""VS3 gate inventory. Execution and external acceptance remain separate."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
RUST_TESTS=162
PROCESS_TESTS=24
WORKER_TESTS=6

def sources():
 paths=[]
 for directory in ['apps/desktop-qt/vslice-003','apps/desktop-qt/vslice-003-tests']:
  paths.extend(str(p.relative_to(ROOT)) for p in (ROOT/directory).rglob('*') if p.is_file() and p.suffix in {'.rs','.cpp','.h','.toml','.lock','.pro','.py','.md','.qml','.qrc'})
 paths += ['apps/desktop-qt/validate-vslice-003.sh','.github/workflows/vslice-003.yml','docs/implementation/vslice-003/evidence/verify.py']
 paths += ['docs/implementation/vslice-003/'+n+'.md' for n in ['TEST_MATRIX','VALIDATION_TASK','REVIEW_CHECKLISTS']]
 return sorted(paths)
