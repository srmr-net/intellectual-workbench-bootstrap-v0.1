"""Collect executed VS3 facts; independent hosted verification is still required."""
from pathlib import Path
import hashlib,json,os,re,shutil,subprocess
from contract import ROOT,PROCESS_TESTS,WORKER_TESTS,sources
OUT=ROOT/'target/abi-evidence/vs3'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def test_vs3_025_source_bound_independent_reviews():
 reviews=[]
 for role in ['agent-security','qt-kernel']:
  path=ROOT/f'docs/implementation/vslice-003/evidence/review-{role}.json';r=json.loads(path.read_text())
  assert r['status']=='APPROVED' and not r['remainingMandatory'] and r['reviewer']!='/root'
  subprocess.run(['git','merge-base','--is-ancestor',r['implementationCommit'],'HEAD'],cwd=ROOT,check=True)
  for name in sources():assert r['sourceSha256'][name]==sha(ROOT/name),(role,name)
  assert r['independentTests']
  for case in r['independentTests']:
   assert case['exitCode']==0 and case.get('ignoredOrSkipped',0)==0
   log=ROOT/case['log'];assert sha(log)==case['logSha256'],str(log)
   dest=OUT/'reviewer-logs'/role/log.name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(log,dest)
  shutil.copyfile(path,OUT/f'review-{role}.json');reviews.append(r)
 assert reviews[0]['reviewer']!=reviews[1]['reviewer']
 print('test_vs3_025_source_bound_independent_reviews ... ok')
 return reviews

def main():
 for path,n in [('process-tests.txt',PROCESS_TESTS),('worker-tests.txt',WORKER_TESTS)]:
  text=(OUT/path).read_text();assert f'Ran {n} tests' in text and '\nOK\n' in text and not re.search(r'FAILED|skipped|ERROR:',text),path
 tests=re.findall(r'^(test_vs3_\d{3}_\w+) .*? \.\.\. ok$',(OUT/'process-tests.txt').read_text(),re.M)
 assert len(tests)==24 and {t[9:12] for t in tests}=={f'{n:03d}' for n in range(1,25)}
 reviews=test_vs3_025_source_bound_independent_reviews()
 for name in sources():
  dest=OUT/'sources'/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
 reports=[(p,json.loads(p.read_text())) for p in sorted((OUT/'process').rglob('report.json'))]
 assert len(reports)==25,len(reports)
 assert len({r['pid'] for _,r in reports})==len({r['workerProcessPid'] for _,r in reports})==25
 app=ROOT/'target/vs3-build/workbench-vslice003';worker=ROOT/'target/debug/wb-vslice003-worker'
 shutil.copyfile(app,OUT/app.name);shutil.copyfile(worker,OUT/worker.name)
 for p,r in reports:assert not r['failure'] and r['exitCode']==0 and r['guiThread'] and r['pid']!=r['workerProcessPid']==r['finalState']['pid'] and r['binarySha256']==sha(app) and r['initialProviderCalls']=='0' and r['initialGuestInvocations']=='0',str(p)
 def case(number):return [(p,r) for p,r in reports if f'test_vs3_{number:03d}_' in str(p)]
 happy=case(18)[0][1];assert happy['finalState']['projection']['values']['payment']=='1753.77'
 assert next(s for s in happy['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='1'
 first,last=[r for _,r in case(22)];assert first['pid']!=last['pid'] and first['workerProcessPid']!=last['workerProcessPid']
 assert first['finalState']['rooms']==last['finalState']['rooms'] and last['finalState']['invocations']=='0'
 assert next(s for s in last['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='0'
 assert 'OFFLINE_IPV4_IPV6_DENIED' in case(22)[1][0].with_name('stdout.txt').read_text()
 two=case(21)[0][1];assert len(two['finalState']['rooms'])==2
 assert next(s for s in two['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='2'
 run=json.loads((OUT.parent/'run.json').read_text())
 result={'source':run['headSha'],'trackedChanges':run['trackedChanges'],'hostedExecution':os.environ.get('GITHUB_ACTIONS')=='true','cases':{f'VS3-{n:03d}':{'test':next(t for t in tests if t.startswith(f'test_vs3_{n:03d}_')),'status':'executed'} for n in range(1,25)},'reviewers':[r['reviewer'] for r in reviews],'processTests':24,'workerTests':WORKER_TESTS,'appWorkerPairs':25,'appSha256':sha(app),'workerSha256':sha(worker),'rustRegressionTests':162,'coldProviderCalls':0,'coldGuestInvocations':0,'acceptance':'Proposed until successful hosted run and separate downloaded verification'}
 result['cases']['VS3-025']={'test':'test_vs3_025_source_bound_independent_reviews','status':'executed'}
 result['cases']['VS3-026']={'test':'test_vs3_026_complete_downloaded_hosted_evidence','status':'requires separate completed hosted run verification'}
 result['sha256']={str(p.relative_to(OUT)):sha(p) for p in sorted(OUT.rglob('*')) if p.is_file() and p!=OUT/'report.json'}
 (OUT/'report.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='sha256'},indent=2))
if __name__=='__main__':main()
