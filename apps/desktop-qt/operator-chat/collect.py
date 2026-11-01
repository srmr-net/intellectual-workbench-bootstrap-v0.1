"""Source-bound deterministic chat evidence; never inference or conversation logging."""
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
ROOT = Path(__file__).resolve().parents[3]
BASE = '77351e46cb0634988f4be37edd5a572c18c7d656'
OUT = ROOT / 'target/chat-001-evidence'
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
def sources():
    names = {str(p.relative_to(ROOT)) for p in (ROOT/'apps/desktop-qt/operator-chat').rglob('*') if p.is_file() and not {'target','__pycache__'}.intersection(p.parts) and p.suffix != '.pyc'}
    names.update(str(p.relative_to(ROOT)) for p in (ROOT/'crates/wb-contracts/src').rglob('*.rs'))
    names.update(['workbench','operator-chat','WORKBENCH.md','Cargo.toml','crates/wb-contracts/Cargo.toml','.github/workflows/operator-chat.yml','docs/adr/ADR-016.md','docs/implementation/operator-chat/TASK.md',*(f'apps/desktop-qt/live-provider/host/{p}' for p in ['src/policy.rs','src/transport.rs','transport.py'])])
    return sorted(names)
def source_map():
    return {p:sha(ROOT/p) for p in sources()}
def main():
    protected=['crates','schemas','ontology','references','apps/desktop-qt/desktop','apps/desktop-qt/vslice-003','apps/desktop-qt/vslice-004','apps/desktop-qt/live-provider','apps/desktop-qt/operator-direct']
    assert not subprocess.check_output(['git','diff',BASE,'--name-only','--',*protected],cwd=ROOT,text=True),'accepted implementation changed'
    qt=(OUT/'qt-tests.txt').read_text()
    assert re.search(r'Totals: 9 passed, 0 failed, 0 skipped',qt) and 'FAIL!' not in qt
    native=(OUT/'native-tests.txt').read_text()
    assert '16 passed; 0 failed; 0 ignored' in native
    process=(OUT/'process-tests.txt').read_text()
    assert 'Ran 3 tests' in process and re.search(r'^OK$',process,re.M)
    inventory=source_map()
    receipts=[]
    for role in ['ux','security']:
        path=ROOT/f'docs/implementation/operator-chat/evidence/review-{role}.json'
        review=json.loads(path.read_text())
        assert review['role']==role and review['status']=='APPROVED' and not review['remainingMandatory'] and review['reviewer']!='/root'
        assert review['sourceSha256']==inventory
        subprocess.run(['git','merge-base','--is-ancestor',review['implementationCommit'],'HEAD'],cwd=ROOT,check=True)
        assert review['independentTests']
        for test in review['independentTests']:
            assert test['exitCode']==0 and test.get('ignoredOrSkipped',0)==0
            log=ROOT/test['log'];assert sha(log)==test['logSha256']
            dest=OUT/'reviewer-logs'/role/log.name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(log,dest)
        shutil.copyfile(path,OUT/path.name);receipts.append(review)
    assert len({r['reviewer'] for r in receipts})==2
    for p in sources():
        dest=OUT/'sources'/p;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/p,dest)
    result={'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'trackedChanges':subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True),'sourceSha256':inventory,'hosted':os.environ.get('GITHUB_ACTIONS')=='true','runId':os.environ.get('GITHUB_RUN_ID'),'runAttempt':os.environ.get('GITHUB_RUN_ATTEMPT'),'workflow':os.environ.get('GITHUB_WORKFLOW'),'ref':os.environ.get('GITHUB_REF'),'runnerOS':os.environ.get('RUNNER_OS'),'nativeTests':16,'qtCases':7,'processTests':3,'live':'not performed by deterministic tests','binarySha256':{p:sha(ROOT/p) for p in ['target/operator-chat-native/debug/wb-operator-chat','target/operator-chat-fixture/debug/wb-operator-chat','target/operator-chat-build/workbench-operator-chat','target/operator-chat-tests/operator-chat-tests']}}
    result['evidenceSha256']={str(p.relative_to(OUT)):sha(p) for p in OUT.rglob('*') if p.is_file() and p.name!='report.json'}
    (OUT/'report.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ['sourceSha256','evidenceSha256']},indent=2))
if __name__=='__main__':main()
