#!/usr/bin/env python3
"""LLM-024: independently verify downloaded archive, retained regressions and hosted identity."""
import argparse,hashlib,json,re,subprocess,sys
from pathlib import Path
from zipfile import ZipFile
ROOT=Path(__file__).resolve().parents[4]
def digest(b):return hashlib.sha256(b).hexdigest()
def validate_hosted_run(api,run,report,source,head):
 g=run['github']
 assert api['status']=='completed' and api['conclusion']=='success' and api['event']=='pull_request' and api['head_sha']==head
 assert api['path']=='.github/workflows/adr-006.yml'
 assert str(api['id'])==report['runId']==g['GITHUB_RUN_ID'] and str(api['run_attempt'])==report['runAttempt']==g['GITHUB_RUN_ATTEMPT']
 assert api['repository']['full_name']==g['GITHUB_REPOSITORY']=='srmr-net/intellectual-workbench-bootstrap-v0.1'
 assert g['GITHUB_SHA']==source and g['RUNNER_OS']=='Linux' and report['hostedExecution']
 assert g['GITHUB_REF'] in {f"refs/pull/{p['number']}/merge" for p in api['pull_requests']}

def verify(a):
 assert digest(a.archive.read_bytes())==a.sha256
 # Retained verifier stays byte-identical. Its own workflow identity is not
 # applicable to the additive outer workflow; validate its deterministic chain,
 # then independently bind every retained source/run fact to this outer run.
 command=[sys.executable,str(ROOT/'docs/implementation/vslice-003/evidence/verify.py'),str(a.archive),'--sha256',a.sha256,'--source',a.source]
 retained=json.loads(subprocess.check_output(command,text=True,cwd=ROOT))
 with ZipFile(a.archive) as z:
  load=lambda n:json.loads(z.read(n));report=load('llm/report.json');run=load('run.json')
  assert report['source']==run['headSha']==a.source and not run['trackedChanges']
  assert set(report['cases'])=={f'LLM-{n:03d}' for n in range(1,24)}
  for name,h in report['sha256'].items():assert digest(z.read('llm/'+name))==h,name
  for name,count in [('transport-tests.txt',4),('process-tests.txt',7),('metadata-tests.txt',2)]:
   text=z.read('llm/'+name).decode();assert f'Ran {count} tests' in text and '\nOK\n' in text and not re.search(r'FAILED|skipped|ERROR:',text)
  for name in ['default-tests.txt','mock-tests.txt']:
   text=z.read('llm/'+name).decode();assert '24 passed; 0 failed; 0 ignored' in text and 'dropping_pending_process_exchange_kills_and_reaps_child ... ok' in text and 'FAILED' not in text
  reviews=[load('llm/review-'+role+'.json') for role in ['security','integration']]
  assert len({r['reviewer'] for r in reviews})==2
  for role,r in zip(['security','integration'],reviews):
   assert r['status']=='APPROVED' and r['reviewer']!='/root' and not r['remainingMandatory']
   for name,h in r['sourceSha256'].items():assert digest(z.read('llm/sources/'+name))==h
   assert r['independentTests']
   for case in r['independentTests']:
    assert case['exitCode']==0 and case.get('ignoredOrSkipped',0)==0
    assert digest(z.read('llm/reviewer-logs/'+role+'/'+Path(case['log']).name))==case['logSha256']
  reports=[(name,load(name)) for name in z.namelist() if name.startswith('llm/process/') and name.endswith('/report.json')]
  assert len(reports)==14 and len({r['pid'] for _,r in reports})==14 and len({r['workerProcessPid'] for _,r in reports})==14
  for name,r in reports:assert not r['failure'] and r['exitCode']==0 and r['guiThread'] and r['pid']!=r['workerProcessPid']==r['finalState']['pid'] and r['binarySha256']==report['appSha256'] and r['initialProviderCalls']=='0' and r['initialGuestInvocations']=='0',name
  cold=sorted((n,r) for n,r in reports if '018_019_' in n);assert len(cold)==2
  first,last=[r for _,r in cold];assert first['finalState']['rooms']==last['finalState']['rooms'] and first['finalState']['projection']['values']['payment']==last['finalState']['projection']['values']['payment']=='1753.77'
  assert next(s for s in first['samples'] if s['action']=='operatorStatus')['providerState']['remoteAttempts']=='1'
  assert next(s for s in last['samples'] if s['action']=='operatorStatus')['providerState']['remoteAttempts']=='0' and last['finalState']['invocations']=='0'
  assert 'OFFLINE_IPV4_IPV6_DENIED' in z.read(cold[1][0].replace('report.json','stdout.txt')).decode()
  if a.hosted:
   validate_hosted_run(json.loads(a.run_metadata.read_text()),run,report,a.source,a.head)
  print(json.dumps({'verified':True,'gate':'LLM-024','hosted':a.hosted,'source':a.source,'head':a.head,'archiveSha256':a.sha256,'nativeConformanceTests':24,'transportTests':4,'qtProcessTests':7,'appWorkerPairs':14,'retained':retained,'live':'separate required evidence'},indent=2))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--sha256',required=True);p.add_argument('--source',required=True);p.add_argument('--hosted',action='store_true');p.add_argument('--run-metadata',type=Path);p.add_argument('--head');verify(p.parse_args())
