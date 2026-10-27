#!/usr/bin/env python3
"""ADR005 foundation regression verification ONLY; not OP acceptance. Workflow conclusion checked separately."""
import argparse,hashlib,json,re
from pathlib import Path
from zipfile import ZipFile
p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--sha256',required=True);p.add_argument('--source',required=True);p.add_argument('--hosted',action='store_true');a=p.parse_args()
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(a.archive.read_bytes())==a.sha256
with ZipFile(a.archive) as z:
 load=lambda n:json.loads(z.read(n))
 run=load('run.json');assert run['headSha']==a.source and not run['trackedChanges'];assert all(v=='passed' for v in run['checks'].values())
 for name,digest in run['sha256'].items():assert sha(z.read(name))==digest,name
 r=load('vs2/report.json');assert r['headSha']==a.source and not r['trackedChanges'];assert set(r['cases'])=={f'VS2-{n:03d}' for n in range(1,25)}
 logs=z.read('workspace-tests.txt').decode();count=sum(map(int,re.findall(r'test result: ok\. (\d+) passed',logs)));assert count==r['rustTestsPassed']==147,count
 assert not re.search(r'test \S+ \.\.\. (?:FAILED|ignored)',logs)
 foundation=['deterministic_fixture_and_explicit_units','no_defaults_or_injected_instructions','strict_untrusted_json','ambiguity_cancel_unavailable_and_deadline','cancellation_and_deadline_rechecked_after_ready','real_admission_preview_approval_ingress_and_replay','checkpoint_failure_is_honest_partial_completion','fresh_preview_identity_stale_revision_and_expiry','failed_apply_removes_only_new_execution_consent']
 for name in foundation:assert f'test {name} ... ok' in logs,name

 for name,n in [('registry/report.json',20),('vslice/report.json',20),('ui/report.json',20)]:assert len(load(name)['cases'])==n
 ipc=load('ipc-report.json');assert len(ipc['cases'])==15 and len(ipc['abiCases'])==10
 numerical=load('vslice/numerical-results.json');assert len(numerical)==33 and all(x['actual']==x['expected'] for x in numerical)
 qt=z.read('vs2/qt-tests.txt').decode();assert '8 passed, 0 failed, 0 skipped' in qt and not re.search(r'FAIL!|SKIP\s+:',qt)
 process=z.read('vs2/process-tests.txt').decode();assert 'Ran 14 tests' in process and '\nOK\n' in process and 'FAILED' not in process
 for scope in ['qt','b']:
  review=load('vs2/review-'+scope+'.json');assert review['status']=='APPROVED' and review['independentTests'] and review['implementationCommit']=='615d76b54ac415bc1d9dd85b7237bfba3f035c68'
  for name,digest in review['sourceSha256'].items():assert sha(z.read('vs2/sources/'+name))==digest,name
 app=sha(z.read('vs2/workbench-desktop'));assert app==r['appSha256']
 reports=[(name,load(name)) for name in z.namelist() if name.startswith('vs2/process/') and name.endswith('/report.json')];assert len(reports)==25
 assert len({x['pid'] for _,x in reports})==25 and len({x['workerProcessPid'] for _,x in reports})==25
 for name,x in reports:
  assert x['exitCode']==0 and not x['failure'] and x['guiThread'] and x['binarySha256']==app,name
  assert x['pid']!=x['workerProcessPid']==x['finalState']['pid'],name
 cold=[x for name,x in reports if 'test_vs2_018_' in name];assert len(cold)==2 and cold[0]['pid']!=cold[1]['pid'];assert cold[1]['finalState']['invocations']=='0';assert cold[0]['finalState']['rooms']==cold[1]['finalState']['rooms'];assert cold[1]['finalState']['projection']['values']['payment']=='1461.48'
 offline=[name for name,_ in reports if 'test_vs2_022_' in name and '/run-2/' in name];assert len(offline)==1;assert 'OFFLINE_IPV4_IPV6_DENIED' in z.read(offline[0].replace('report.json','stdout.txt')).decode()
 frozen={'component.wit':'cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0','registry/component.schema.json':'15f2b7a2b84655e62f14fc521fe06a291fd00bf5fbcc120e509aaa84e9e1dd55','registry/ontology.ttl':'1741a428b7ca8bd9d4a761659c30bc5f914fcac60d201ad5361cc54ca9cc3bda','ui/Main.qml':'ed1533ed44081806e598b533a4f5f9777ccd65fa838d462ea4059ca7920ab13f'}
 for name,digest in frozen.items():assert sha(z.read(name))==digest,name
 if a.hosted:assert r['hostedExecution'] and run['github']['RUNNER_OS']=='Linux' and run['github']['GITHUB_SHA']==a.source
 print(json.dumps({'verified':True,'operatorAcceptance':False,'operatorFoundationTests':9,'source':a.source,'archiveSha256':a.sha256,'recordedHashes':len(run['sha256']),'rustTests':count,'processTests':14,'desktopProcesses':25,'qtSummary':r['qtSummary'],'coldProcessIds':r['coldProcessIds'],'coldWorkerIds':r['coldWorkerIds'],'coldRestoreInvocations':0,'appSha256':app,'qtVersions':z.read('vs2/qt-version.txt').decode()},indent=2))
