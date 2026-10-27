#!/usr/bin/env python3
"""VS3-026: independently verify unmodified downloaded ZIP and retained OP gates."""
import argparse,hashlib,json,re,subprocess,sys
from pathlib import Path
from zipfile import ZipFile
ROOT=Path(__file__).resolve().parents[4]
p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--sha256',required=True);p.add_argument('--source',required=True);p.add_argument('--hosted',action='store_true');p.add_argument('--run-metadata',type=Path);p.add_argument('--head')
def validate_hosted_run(api,run,report,source,head):
 assert api['status']=='completed' and api['conclusion']=='success' and api['event']=='pull_request' and api['head_sha']==head
 assert api['path']=='.github/workflows/vslice-003.yml'
 github=run['github']
 assert str(api['id'])==github['GITHUB_RUN_ID'] and str(api['run_attempt'])==github['GITHUB_RUN_ATTEMPT']
 assert github['GITHUB_REPOSITORY']==api['repository']['full_name']=='srmr-net/intellectual-workbench-bootstrap-v0.1'
 assert github['GITHUB_SHA']==run['headSha']==report['source']==source and github['RUNNER_OS']=='Linux' and report['hostedExecution']
 assert github['GITHUB_REF'] in {f"refs/pull/{pr['number']}/merge" for pr in api['pull_requests']}

def test_vs3_026_complete_downloaded_hosted_evidence(a):
 sha=lambda b:hashlib.sha256(b).hexdigest()
 assert sha(a.archive.read_bytes())==a.sha256
 command=[sys.executable,str(ROOT/'docs/implementation/adr-005/evidence/verify.py'),str(a.archive),'--sha256',a.sha256,'--source',a.source]
 if a.hosted:command+=['--hosted','--run-metadata',str(a.run_metadata),'--head',a.head]
 baseline=subprocess.run(command,capture_output=True,text=True,check=True);retained=json.loads(baseline.stdout)
 with ZipFile(a.archive) as z:
  load=lambda n:json.loads(z.read(n))
  r=load('vs3/report.json');assert r['source']==a.source and not r['trackedChanges']
  assert len(r['cases'])==26 and set(r['cases'])=={f'VS3-{n:03d}' for n in range(1,27)}
  for name,digest in r['sha256'].items():assert sha(z.read('vs3/'+name))==digest,name
  for name,n in [('process-tests.txt',24),('worker-tests.txt',6),('metadata-tests.txt',6)]:
   text=z.read('vs3/'+name).decode();assert f'Ran {n} tests' in text and '\nOK\n' in text and not re.search(r'FAILED|skipped|ERROR:',text)
  process=z.read('vs3/process-tests.txt').decode()
  for n in range(1,25):assert re.search(r'^test_vs3_'+f'{n:03d}'+r'_\w+ .*? \.\.\. ok$',process,re.M),n
  reviews=[load('vs3/review-'+role+'.json') for role in ['agent-security','qt-kernel']]
  assert len({v['reviewer'] for v in reviews})==2 and all(v['reviewer']!='/root' and v['status']=='APPROVED' and not v['remainingMandatory'] for v in reviews)
  for review in reviews:
   for name,digest in review['sourceSha256'].items():assert sha(z.read('vs3/sources/'+name))==digest,name
   for case in review['independentTests']:
    assert case['exitCode']==0 and case.get('ignoredOrSkipped',0)==0
    role='agent-security' if review is reviews[0] else 'qt-kernel'
    assert sha(z.read('vs3/reviewer-logs/'+role+'/'+Path(case['log']).name))==case['logSha256']
  reports=[(name,load(name)) for name in z.namelist() if name.startswith('vs3/process/') and name.endswith('/report.json')];assert len(reports)==25
  assert len({r['pid'] for _,r in reports})==len({r['workerProcessPid'] for _,r in reports})==25
  app=sha(z.read('vs3/workbench-vslice003'));assert app==r['appSha256'];assert sha(z.read('vs3/wb-vslice003-worker'))==r['workerSha256']
  for name,v in reports:assert not v['failure'] and v['exitCode']==0 and v['guiThread'] and v['pid']!=v['workerProcessPid']==v['finalState']['pid'] and v['binarySha256']==app and v['initialProviderCalls']=='0' and v['initialGuestInvocations']=='0',name
  def case(n):return sorted((name,v) for name,v in reports if f'test_vs3_{n:03d}_' in name)
  first,last=[v for _,v in case(22)];assert first['pid']!=last['pid'] and first['workerProcessPid']!=last['workerProcessPid'] and first['finalState']['rooms']==last['finalState']['rooms'];assert last['finalState']['invocations']=='0'
  assert next(s for s in last['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='0'
  assert 'OFFLINE_IPV4_IPV6_DENIED' in z.read(case(22)[1][0].replace('report.json','stdout.txt')).decode()
  two=case(21)[0][1];assert len(two['finalState']['rooms'])==2 and next(s for s in two['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='2'
  happy=case(18)[0][1];assert happy['finalState']['projection']['values']['payment']=='1753.77' and next(s for s in happy['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='1'
  for name,v in case(6):assert v['finalState']['projection']['values']['payment']=='1461.48'
  if a.hosted:
   api=json.loads(a.run_metadata.read_text());validate_hosted_run(api,load('run.json'),r,a.source,a.head)
  print(json.dumps({'verified':True,'hosted':a.hosted,'source':a.source,'head':a.head,'archiveSha256':a.sha256,'test':'test_vs3_026_complete_downloaded_hosted_evidence','vs3Gates':26,'namedJourneyTests':24,'nativeWorkerTests':6,'metadataTests':6,'appWorkerPairs':25,'retained':retained,'appSha256':app,'workerSha256':r['workerSha256'],'coldProviderCalls':0,'coldGuestInvocations':0,'qt':z.read('vs3/qt-version.txt').decode()},indent=2))

if __name__=="__main__":test_vs3_026_complete_downloaded_hosted_evidence(p.parse_args())
