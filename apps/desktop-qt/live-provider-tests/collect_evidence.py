"""Deterministic evidence. LLM-024 additionally requires completed hosted download verification."""
import json,os,re,shutil,subprocess
from pathlib import Path
from contract import ROOT,BASE,sources,sha
OUT=Path(os.environ.get('WB_LLM_EVIDENCE_DIR',ROOT/'target/abi-evidence/llm'))
def reviews():
 receipts=[]
 for role in ['security','integration']:
  path=ROOT/f'docs/implementation/adr-006/evidence/review-{role}.json';r=json.loads(path.read_text())
  assert r['role']==role
  assert r['status']=='APPROVED' and not r['remainingMandatory'] and r['reviewer']!='/root'
  subprocess.run(['git','merge-base','--is-ancestor',r['implementationCommit'],'HEAD'],cwd=ROOT,check=True)
  assert r['sourceSha256']=={name:sha(ROOT/name) for name in sources()},role
  assert r['independentTests']
  for case in r['independentTests']:
   assert case['exitCode']==0 and case.get('ignoredOrSkipped',0)==0
   log=ROOT/case['log'];assert sha(log)==case['logSha256'];dest=OUT/'reviewer-logs'/role/log.name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(log,dest)
  shutil.copyfile(path,OUT/path.name);receipts.append(r)
 assert receipts[0]['reviewer']!=receipts[1]['reviewer']
 return receipts
def main():
 for log,count in [('transport-tests.txt',5),('process-tests.txt',7),('metadata-tests.txt',2)]:
  text=(OUT/log).read_text();assert f'Ran {count} tests' in text and '\nOK\n' in text and not re.search(r'FAILED|skipped|ERROR:',text),log
 for log in ['default-tests.txt','mock-tests.txt']:
  text=(OUT/log).read_text();assert '27 passed; 0 failed; 0 ignored' in text and 'dropping_pending_process_exchange_kills_and_reaps_child ... ok' in text and 'FAILED' not in text,log
 # Every accepted tracked file is byte-preserved except explicitly allocated bookkeeping.
 changed=subprocess.check_output(['git','diff',BASE,'--name-only'],cwd=ROOT,text=True).splitlines()
 allowed={'docs/CONTRACT_STATUS.md','docs/implementation/BACKLOG.md','docs/adr/ADR-006.md'}
 prior=set(subprocess.check_output(['git','ls-tree','-r','--name-only',BASE],cwd=ROOT,text=True).splitlines())
 assert not [p for p in changed if p in prior and p not in allowed],changed
 receipts=reviews()
 for name in sources():
  dest=OUT/'sources'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
 reports=[json.loads(p.read_text()) for p in sorted((OUT/'process').rglob('report.json'))]
 assert len(reports)==14,len(reports)
 app=ROOT/'target/llm-qt-build/workbench-live-provider';worker=ROOT/'target/debug/wb-live-worker'
 for r in reports:assert r['exitCode']==0 and not r['failure'] and r['guiThread'] and r['pid']!=r['workerProcessPid']==r['finalState']['pid'] and r['binarySha256']==sha(app) and r['initialProviderCalls']=='0' and r['initialGuestInvocations']=='0'
 assert len({r['pid'] for r in reports})==14 and len({r['workerProcessPid'] for r in reports})==14
 revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
 result={'source':revision,'hostedExecution':os.environ.get('GITHUB_ACTIONS')=='true','runId':os.environ.get('GITHUB_RUN_ID'),'runAttempt':os.environ.get('GITHUB_RUN_ATTEMPT'),'workflow':os.environ.get('GITHUB_WORKFLOW'),'cases':{f'LLM-{n:03d}':{'status':'executed; see CONFORMANCE.md'} for n in range(1,24)},'LLM-024':'requires successful completed hosted run and independent downloaded verification','live':'NOT RUN by deterministic wrapper','reviewers':[r['reviewer'] for r in receipts],'nativeConformanceTests':27,'transportTests':5,'qtProcessTests':7,'appWorkerPairs':14,'appSha256':sha(app),'workerSha256':sha(worker),'acceptedBase':BASE}
 result['sha256']={str(p.relative_to(OUT)):sha(p) for p in sorted(OUT.rglob('*')) if p.is_file() and p!=OUT/'report.json'}
 (OUT/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='sha256'},indent=2))
if __name__=='__main__':main()
