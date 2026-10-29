#!/usr/bin/env python3
"""Verify downloaded VS4 archive, retained ADR006 chain and final-source identity."""
import argparse,hashlib,importlib.util,json,re,subprocess,sys
from pathlib import Path
from zipfile import ZipFile
ROOT=Path(__file__).resolve().parents[4]
spec=importlib.util.spec_from_file_location('vs4_contract',ROOT/'apps/desktop-qt/vslice-004-tests/contract.py');contract=importlib.util.module_from_spec(spec);spec.loader.exec_module(contract)
def digest(b):return hashlib.sha256(b).hexdigest()
def validate_source_map(actual,expected):assert actual==expected,'source inventory/hash differs from Git commit'
def validate_hosted_run(api,run,report,source,head):
 g=run['github'];assert api['status']=='completed' and api['conclusion']=='success' and api['event']=='pull_request' and api['head_sha']==head
 assert api['path']=='.github/workflows/vslice-004.yml'
 assert str(api['id'])==report['runId']==g['GITHUB_RUN_ID'] and str(api['run_attempt'])==report['runAttempt']==g['GITHUB_RUN_ATTEMPT']
 assert api['repository']['full_name']==g['GITHUB_REPOSITORY']=='srmr-net/intellectual-workbench-bootstrap-v0.1'
 assert g['GITHUB_SHA']==source and g['RUNNER_OS']=='Linux' and report['hostedExecution']
 assert g['GITHUB_REF'] in {f"refs/pull/{p['number']}/merge" for p in api['pull_requests']}
def expected_sources(source):
 assert re.fullmatch(r'[0-9a-f]{40}',source)
 paths=subprocess.check_output(['git','ls-tree','-r','--name-only',source],cwd=ROOT,text=True).splitlines()
 return {p:digest(subprocess.check_output(['git','show',source+':'+p],cwd=ROOT)) for p in paths if contract.selected(p)}
def verify(a):
 assert digest(a.archive.read_bytes())==a.sha256
 retained=json.loads(subprocess.check_output([sys.executable,str(ROOT/'docs/implementation/adr-006/evidence/verify.py'),str(a.archive),'--sha256',a.sha256,'--source',a.source],cwd=ROOT,text=True));expected=expected_sources(a.source)
 with ZipFile(a.archive) as z:
  assert len(z.namelist())==len(set(z.namelist()))
  load=lambda n:json.loads(z.read(n));r=load('vs4/report.json');run=load('run.json');assert r['source']==run['headSha']==a.source and not run['trackedChanges']
  assert set(r['cases'])=={f'VS4-{n:03d}' for n in range(1,27)}
  for n,h in r['sha256'].items():assert digest(z.read('vs4/'+n))==h,n
  text=z.read('vs4/process-tests.txt').decode();assert 'Ran 26 tests' in text and '\nOK\n' in text and not re.search(r'FAILED|skipped|ERROR:',text)
  receipts=[load('vs4/review-'+role+'.json') for role in ['security','desktop']];assert len({v['reviewer'] for v in receipts})==2
  for role,v in zip(['security','desktop'],receipts):
   assert v['role']==role and v['status']=='APPROVED' and not v['remainingMandatory'] and v['reviewer']!='/root';validate_source_map(v['sourceSha256'],expected)
   subprocess.run(['git','merge-base','--is-ancestor',v['implementationCommit'],a.source],cwd=ROOT,check=True)
   for n,h in expected.items():assert digest(z.read('vs4/sources/'+n))==h
   assert v['independentTests']
   for t in v['independentTests']:assert t['exitCode']==0 and t.get('ignoredOrSkipped',0)==0 and digest(z.read('vs4/reviewer-logs/'+role+'/'+Path(t['log']).name))==t['logSha256']
  reports=[load(n) for n in z.namelist() if n.startswith('vs4/process/') and n.endswith('/report.json')];assert len(reports)==r['appWorkerPairs']>=40
  assert len({v['pid'] for v in reports})==len(reports) and len({v['workerProcessPid'] for v in reports})==len(reports)
  for v in reports:assert v['exitCode']==0 and not v['failure'] and v['guiThread'] and v['pid']!=v['workerProcessPid']==v['finalState']['pid'] and v['binarySha256']==r['appSha256'] and v['initialProviderCalls']=='0' and v['initialGuestInvocations']=='0'
  for n in z.namelist():contract.assert_private_artifact(z.read(n))
  if a.hosted:validate_hosted_run(json.loads(a.run_metadata.read_text()),run,r,a.source,a.head)
  print(json.dumps({'verified':True,'gate':'VS4-029','hosted':a.hosted,'source':a.source,'head':a.head,'archiveSha256':a.sha256,'appWorkerPairs':len(reports),'retained':retained,'live':'separate required gate'},indent=2))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('archive',type=Path);p.add_argument('--sha256',required=True);p.add_argument('--source',required=True);p.add_argument('--hosted',action='store_true');p.add_argument('--run-metadata',type=Path);p.add_argument('--head');verify(p.parse_args())
