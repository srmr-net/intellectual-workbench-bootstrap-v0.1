#!/usr/bin/env python3
"""Verify immutable UX001 evidence, retained regressions and hosted source identity."""
import argparse,hashlib,json,re,subprocess,sys
from pathlib import Path
from zipfile import ZipFile
from source_contract import ROOT,selected

def digest(b):return hashlib.sha256(b).hexdigest()
def expected(source):
 assert re.fullmatch('[0-9a-f]{40}',source)
 paths=subprocess.check_output(['git','ls-tree','-r','--name-only',source],cwd=ROOT,text=True).splitlines()
 return {n:digest(subprocess.check_output(['git','show',source+':'+n],cwd=ROOT)) for n in paths if selected(n)}
def hosted(api,run,report,source,head):
 g=run['github'];assert api['status']=='completed' and api['conclusion']=='success' and api['event']=='pull_request' and api['head_sha']==head
 assert api['path']=='.github/workflows/ux-001.yml'
 assert str(api['id'])==report['runId']==g['GITHUB_RUN_ID'] and str(api['run_attempt'])==report['runAttempt']==g['GITHUB_RUN_ATTEMPT']
 assert api['repository']['full_name']==g['GITHUB_REPOSITORY']=='srmr-net/intellectual-workbench-bootstrap-v0.1'
 assert g['GITHUB_SHA']==source and g['RUNNER_OS']=='Linux' and report['hostedExecution']
 assert g['GITHUB_REF'] in {f"refs/pull/{p['number']}/merge" for p in api['pull_requests']}
def verify(a):
 assert digest(a.archive.read_bytes())==a.sha256
 retained=json.loads(subprocess.check_output([sys.executable,str(ROOT/'docs/implementation/vslice-004/evidence/verify.py'),str(a.archive),'--sha256',a.sha256,'--source',a.source],cwd=ROOT,text=True))
 with ZipFile(a.archive) as z:
  assert len(z.namelist())==len(set(z.namelist()));load=lambda n:json.loads(z.read(n));r=load('ux001/report.json');run=load('run.json');assert r['source']==run['headSha']==a.source and not run['trackedChanges']
  assert {c.split('_')[1] for c in r['cases']}=={f'{n:03}' for n in range(1,10)} and len(set(r['cases']))==len(r['cases'])
  for n,h in r['sha256'].items():assert digest(z.read('ux001/'+n))==h,n
  text=z.read('ux001/tests.txt').decode();assert re.search(r'Totals: \d+ passed, 0 failed, 0 skipped',text) and 'FAIL!' not in text
  for case in r['cases']:assert re.search(r'PASS\s*:\s*UxTests::'+re.escape(case)+r'\(\)',text)
  receipts=[load('ux001/review-'+role+'.json') for role in ['ux','security']];assert len({v['reviewer'] for v in receipts})==2
  inventory=expected(a.source)
  for role,v in zip(['ux','security'],receipts):
   assert v['role']==role and v['status']=='APPROVED' and not v['remainingMandatory'] and v['reviewer']!='/root';assert v['sourceSha256']==inventory
   subprocess.run(['git','merge-base','--is-ancestor',v['implementationCommit'],a.source],cwd=ROOT,check=True)
   for n,h in inventory.items():assert digest(z.read('ux001/sources/'+n))==h
   assert v['independentTests']
   for t in v['independentTests']:assert t['exitCode']==0 and t.get('ignoredOrSkipped',0)==0 and digest(z.read('ux001/reviewer-logs/'+role+'/'+Path(t['log']).name))==t['logSha256']
  cold=load('ux001/cold-process-proof.json');assert cold['state']['invocations']=='0' and cold['provider']['remoteAttempts']=='0' and cold['appPid']!=cold['workerPid']
  for n in ['default-workspace.png','components-settings.png','started-canvas.png']:assert z.read('ux001/'+n).startswith(b'\x89PNG')
  if a.hosted:hosted(json.loads(a.run_metadata.read_text()),run,r,a.source,a.head)
  print(json.dumps({'verified':True,'hosted':a.hosted,'source':a.source,'head':a.head,'archiveSha256':a.sha256,'cases':r['cases'],'retained':retained,'live':'separate real walkthrough required'},indent=2))
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('archive',type=Path);p.add_argument('--sha256',required=True);p.add_argument('--source',required=True);p.add_argument('--hosted',action='store_true');p.add_argument('--run-metadata',type=Path);p.add_argument('--head');verify(p.parse_args())
