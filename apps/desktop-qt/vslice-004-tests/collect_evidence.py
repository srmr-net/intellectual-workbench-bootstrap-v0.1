"""Collect successful executed gates, bound sources and independent reviews."""
import json,os,re,shutil,subprocess
from pathlib import Path
from contract import ROOT,BASE,sources,sha,assert_private_artifact
OUT=Path(os.environ.get('WB_VS4_EVIDENCE_DIR',ROOT/'target/abi-evidence/vs4'))
def reviews():
 receipts=[]
 for role in ['security','desktop']:
  path=ROOT/f'docs/implementation/vslice-004/evidence/review-{role}.json';r=json.loads(path.read_text());assert r['role']==role and r['status']=='APPROVED' and not r['remainingMandatory'] and r['reviewer']!='/root'
  subprocess.run(['git','merge-base','--is-ancestor',r['implementationCommit'],'HEAD'],cwd=ROOT,check=True)
  assert r['sourceSha256']=={n:sha(ROOT/n) for n in sources()},role
  assert r['independentTests']
  for case in r['independentTests']:
   assert case['exitCode']==0 and case.get('ignoredOrSkipped',0)==0
   log=ROOT/case['log'];assert sha(log)==case['logSha256'];dest=OUT/'reviewer-logs'/role/log.name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(log,dest)
  shutil.copyfile(path,OUT/path.name);receipts.append(r)
 assert receipts[0]['reviewer']!=receipts[1]['reviewer'];return receipts

def main():
 text=(OUT/'process-tests.txt').read_text();assert 'Ran 26 tests' in text and '\nOK\n' in text and not re.search(r'FAILED|skipped|ERROR:',text)
 names=re.findall(r'^(test_vs4_(\d{3})_\w+) .* \.\.\. ok$',text,re.M);assert {n for _,n in names}=={f'{n:03d}' for n in range(1,27)}
 for name in ['default-tests.txt','mock-tests.txt']:
  text=(OUT/name).read_text();assert '2 passed; 0 failed; 0 ignored' in text and 'FAILED' not in text
 receipts=reviews()
 changed=subprocess.check_output(['git','diff',BASE,'--name-only'],cwd=ROOT,text=True).splitlines();prior=set(subprocess.check_output(['git','ls-tree','-r','--name-only',BASE],cwd=ROOT,text=True).splitlines());assert not [p for p in changed if p in prior and p!='docs/implementation/BACKLOG.md']
 for name in sources():
  dest=OUT/'sources'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
 reports=[json.loads(p.read_text()) for p in (OUT/'process').rglob('report.json')];assert len(reports)>=40
 for r in reports:assert r['exitCode']==0 and not r['failure'] and r['guiThread'] and r['pid']!=r['workerProcessPid']==r['finalState']['pid'] and r['initialProviderCalls']=='0' and r['initialGuestInvocations']=='0' and r['binarySha256']==sha(ROOT/'target/vs4-qt-build/workbench-vslice004')
 assert len({r['pid'] for r in reports})==len(reports) and len({r['workerProcessPid'] for r in reports})==len(reports)
 result={'source':subprocess.check_output(['git','rev-parse','HEAD'],text=True,cwd=ROOT).strip(),'acceptedBase':BASE,'hostedExecution':os.environ.get('GITHUB_ACTIONS')=='true','runId':os.environ.get('GITHUB_RUN_ID'),'runAttempt':os.environ.get('GITHUB_RUN_ATTEMPT'),'workflow':os.environ.get('GITHUB_WORKFLOW'),'cases':{f'VS4-{n}':name for name,n in names},'reviewers':[r['reviewer'] for r in receipts],'appWorkerPairs':len(reports),'appSha256':sha(ROOT/'target/vs4-qt-build/workbench-vslice004'),'workerSha256':sha(ROOT/'target/debug/wb-vslice004-worker'),'live':'Separate supervised gate; not run by CI'}
 result['sha256']={str(p.relative_to(OUT)):sha(p) for p in sorted(OUT.rglob('*')) if p.is_file() and p!=OUT/'report.json'}
 for p in OUT.rglob('*'):
  if p.is_file():assert_private_artifact(p.read_bytes())
 (OUT/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='sha256'},indent=2))
if __name__=='__main__':main()
