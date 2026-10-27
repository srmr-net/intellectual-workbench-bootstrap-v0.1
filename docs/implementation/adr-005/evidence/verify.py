#!/usr/bin/env python3
"""Verify complete ADR005 evidence; no workflow declaration/self-acceptance shortcut."""
import argparse,hashlib,json,re
from pathlib import Path
from zipfile import ZipFile
p=argparse.ArgumentParser(description=__doc__);p.add_argument('archive',type=Path);p.add_argument('--sha256',required=True);p.add_argument('--source',required=True);p.add_argument('--hosted',action='store_true');p.add_argument('--run-metadata',type=Path);p.add_argument('--head');a=p.parse_args()
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(a.archive.read_bytes())==a.sha256
with ZipFile(a.archive) as z:
 load=lambda n:json.loads(z.read(n))
 run=load('run.json');op=load('operator/report.json')
 assert run['headSha']==op['headSha']==a.source and not run['trackedChanges'] and not op['trackedChanges']
 assert all(v=='passed' for v in run['checks'].values())
 for name,digest in run['sha256'].items():assert sha(z.read(name))==digest,name
 for name,digest in op['sha256'].items():assert sha(z.read('operator/'+name))==digest,name
 rust=z.read('workspace-tests.txt').decode();assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',rust)))==op['rustTestsPassed']==162
 assert not re.search(r'test \S+ \.\.\. (?:FAILED|ignored)',rust)
 assert set(op['cases'])=={f'OP-{n:03d}' for n in range(1,25)} and op['newNativeTests']==24
 for n in range(1,23):
  for test in op['cases'][f'OP-{n:03d}']['nativeTests']:assert re.search(r'test (?:\w+::)*'+re.escape(test)+r' \.\.\. ok\b',rust),test
 for path,n in [('registry/report.json',20),('vslice/report.json',20),('ui/report.json',20),('vs2/report.json',24)]:assert len(load(path)['cases'])==n
 ipc=load('ipc-report.json');assert len(ipc['cases'])==15 and len(ipc['abiCases'])==10
 numbers=load('vslice/numerical-results.json');assert len(numbers)==33 and all(r['actual']==r['expected'] for r in numbers)
 for path,n in [('operator/worker-tests.txt',9),('operator/process-tests.txt',6),('vs2/process-tests.txt',14)]:
  text=z.read(path).decode();assert f'Ran {n} tests' in text and '\nOK\n' in text and 'FAILED' not in text and 'skipped' not in text
 for path,n in [('ui/qt-tests.txt',7),('vs2/qt-tests.txt',8)]:
  text=z.read(path).decode();assert f'{n} passed, 0 failed, 0 skipped' in text and not re.search(r'FAIL!|SKIP\s+:',text)
 reviews=[load('operator/review-'+name+'.json') for name in ['agent-security','runtime-integration']]
 assert len({r['reviewer'] for r in reviews})==2 and all(r['reviewer']!='/root' and r['status']=='APPROVED' and r['independentTests'] and not r['remainingMandatory'] for r in reviews)
 for review in reviews:
  for path,digest in review['sourceSha256'].items():assert sha(z.read('operator/sources/'+path))==digest,path
 app=sha(z.read('operator/workbench-operator'));assert app==op['appSha256']
 reports=[(name,load(name)) for name in z.namelist() if name.startswith('operator/process/') and name.endswith('/report.json')];assert len(reports)==7
 assert len({r['pid'] for _,r in reports})==len({r['workerProcessPid'] for _,r in reports})==7
 for name,r in reports:assert not r['failure'] and r['exitCode']==0 and r['guiThread'] and r['pid']!=r['workerProcessPid']==r['finalState']['pid'] and r['binarySha256']==app,name
 happy=next(r for name,r in reports if 'test_op_013_014_017_019_' in name);preview=next(s for s in happy['samples'] if s['action']=='intent');approval=next(s for s in happy['samples'] if s['action']=='approve')
 assert not preview['consentChecked'] and not preview['approveEnabled'] and preview['state']['rooms']==[]
 assert approval['operatorReply']['topologyCommitted'] and approval['operatorReply']['initializationSucceeded'] and approval['operatorReply']['checkpointPersisted'] and approval['operatorReply']['initialInvocations']=='1'
 assert happy['finalState']['projection']['values']['payment']=='1753.77' and next(s for s in happy['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='1'
 cold=sorted([(name,r) for name,r in reports if 'test_op_020_' in name]);assert len(cold)==2
 first,last=[r for _,r in cold];assert first['pid']!=last['pid'] and first['finalState']['rooms']==last['finalState']['rooms'] and last['finalState']['invocations']=='0'
 assert next(s for s in last['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='0' and last['finalState']['projection']['values']['payment']=='1461.48'
 assert 'OFFLINE_IPV4_IPV6_DENIED' in z.read(cold[1][0].replace('report.json','stdout.txt')).decode()
 frozen={'component.wit':'cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0','registry/component.schema.json':'15f2b7a2b84655e62f14fc521fe06a291fd00bf5fbcc120e509aaa84e9e1dd55','registry/ontology.ttl':'1741a428b7ca8bd9d4a761659c30bc5f914fcac60d201ad5361cc54ca9cc3bda','ui/Main.qml':'ed1533ed44081806e598b533a4f5f9777ccd65fa838d462ea4059ca7920ab13f'}
 for name,digest in frozen.items():assert sha(z.read(name))==digest,name
 if a.hosted:
  assert a.run_metadata and a.head
  api=json.loads(a.run_metadata.read_text());assert api['status']=='completed' and api['conclusion']=='success' and api['event']=='pull_request' and api['head_sha']==a.head
  assert api['repository']['full_name']=='srmr-net/intellectual-workbench-bootstrap-v0.1'
  assert op['hostedExecution'] and run['github']['RUNNER_OS']=='Linux' and run['github']['GITHUB_SHA']==a.source
 print(json.dumps({'verified':True,'hosted':a.hosted,'source':a.source,'head':a.head,'archiveSha256':a.sha256,'operatorCases':24,'rustTests':162,'newNativeTests':24,'workerTests':9,'qtProcessTests':6,'desktopProcesses':7,'coldProviderCalls':0,'coldGuestInvocations':0,'appSha256':app,'workerSha256':op['workerSha256'],'reviewers':op['reviewers'],'qtVersions':z.read('operator/qt-version.txt').decode()},indent=2))
