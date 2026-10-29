"""Archive actual executed UX tests, source maps and independent reviews."""
import json,os,re,shutil,subprocess
from pathlib import Path
from source_contract import ROOT,BASE,sha,sources,compatibility
OUT=Path(os.environ.get('WB_UX_EVIDENCE_DIR',ROOT/'target/abi-evidence/ux001'))
def main():
 compatibility();text=(OUT/'tests.txt').read_text();assert re.search(r'Totals: \d+ passed, 0 failed, 0 skipped',text) and 'FAIL!' not in text
 cases=re.findall(r'PASS\s*:\s*UxTests::(ux_\d+_\w+)\(\)',text);assert {c.split("_")[1] for c in cases}=={f"{n:03}" for n in range(1,10)} and len(set(cases))==len(cases)
 ref=(OUT/'reference-tests.txt').read_text();assert '4 passed, 0 failed, 0 skipped' in ref
 assert (OUT/'default-workspace.png').is_file() and (OUT/'components-settings.png').is_file() and (OUT/'started-canvas.png').is_file()
 cold=json.loads((OUT/'cold-process-proof.json').read_text());assert cold['state']['invocations']=='0' and cold['provider']['remoteAttempts']=='0'
 reviewers=[]
 for role in ['ux','security']:
  path=ROOT/f'docs/implementation/ux-001/evidence/review-{role}.json';r=json.loads(path.read_text());assert r['status']=='APPROVED' and r['role']==role and r['reviewer']!='/root' and not r['remainingMandatory'];assert r['sourceSha256']=={p:sha(ROOT/p) for p in sources()};subprocess.run(['git','merge-base','--is-ancestor',r['implementationCommit'],'HEAD'],cwd=ROOT,check=True)
  assert r['independentTests']
  for test in r['independentTests']:
   assert test['exitCode']==0 and test.get('ignoredOrSkipped',0)==0;log=ROOT/test['log'];assert sha(log)==test['logSha256'];dest=OUT/'reviewer-logs'/role/log.name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(log,dest)
  reviewers.append(r['reviewer']);shutil.copyfile(path,OUT/path.name)
 assert len(set(reviewers))==2
 for p in sources():
  dest=OUT/'sources'/p;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/p,dest)
 result={'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'base':BASE,'hostedExecution':os.environ.get('GITHUB_ACTIONS')=='true','runId':os.environ.get('GITHUB_RUN_ID'),'runAttempt':os.environ.get('GITHUB_RUN_ATTEMPT'),'workflow':os.environ.get('GITHUB_WORKFLOW'),'cases':cases,'reviewers':reviewers,'appSha256':sha(ROOT/'target/operator-direct-build/workbench-operator-direct'),'testSha256':sha(ROOT/'target/ux-001-tests/ux-001-tests'),'workerSha256':sha(ROOT/'target/debug/wb-vslice004-worker'),'live':'separate walkthrough required; not run by CI'}
 result['sha256']={str(p.relative_to(OUT)):sha(p) for p in OUT.rglob('*') if p.is_file() and p!=OUT/'report.json'};(OUT/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='sha256'},indent=2))
if __name__=='__main__':main()
