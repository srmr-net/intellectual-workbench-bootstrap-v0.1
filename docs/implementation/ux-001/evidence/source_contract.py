from pathlib import Path
import hashlib,subprocess
ROOT=Path(__file__).resolve().parents[4]
BASE='9c0b27e64dd2d95c46d2f4e1df7bfb83a376dd31'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def selected(name):
 return (name.startswith('apps/desktop-qt/operator-direct/') or name in {'docs/implementation/ux-001/CONFORMANCE.md','workbench','WORKBENCH.md','apps/desktop-qt/build.sh','apps/desktop-qt/run.sh','apps/desktop-qt/validate-ux-001.sh','.github/workflows/ux-001.yml','docs/implementation/ux-001/evidence/source_contract.py','docs/implementation/ux-001/evidence/collect.py','docs/implementation/ux-001/evidence/verify.py','docs/implementation/ux-001/evidence/test_verify.py'}) and not {'target','__pycache__'}.intersection(Path(name).parts)
def sources():
 names={str(p.relative_to(ROOT)) for p in (ROOT/'apps/desktop-qt/operator-direct').rglob('*') if p.is_file() and selected(str(p.relative_to(ROOT)))}
 names|={'docs/implementation/ux-001/CONFORMANCE.md','workbench','WORKBENCH.md','apps/desktop-qt/build.sh','apps/desktop-qt/run.sh','apps/desktop-qt/validate-ux-001.sh','.github/workflows/ux-001.yml',*(f'docs/implementation/ux-001/evidence/{name}.py' for name in ['source_contract','collect','verify','test_verify'])}
 return sorted(names)
def compatibility():
 prior=set(subprocess.check_output(['git','ls-tree','-r','--name-only',BASE],cwd=ROOT,text=True).splitlines());changed=subprocess.check_output(['git','diff',BASE,'--name-only'],cwd=ROOT,text=True).splitlines()
 assert not [n for n in changed if n in prior and not n.startswith('apps/desktop-qt/operator-direct/') and n!='docs/implementation/BACKLOG.md'],'accepted contracts/implementation changed'
