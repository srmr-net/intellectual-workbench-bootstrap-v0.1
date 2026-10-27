"""Executed VS2 evidence; acceptance still requires external hosted verification."""
from pathlib import Path
import hashlib,json,re,shutil,os
ROOT=Path(__file__).resolve().parents[3];REPORT=ROOT/'target/abi-evidence';OUT=REPORT/'vs2'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
run=json.loads((REPORT/'run.json').read_text());rust=(REPORT/'workspace-tests.txt').read_text();qt=(OUT/'qt-tests.txt').read_text();process=(OUT/'process-tests.txt').read_text()
assert all(v=='passed' for v in run['checks'].values())
assert not re.search(r'FAIL!|SKIP\s+:|[1-9]\d* failed|[1-9]\d* skipped',qt)
assert '8 passed, 0 failed, 0 skipped' in qt
assert re.search(r'Ran 14 tests',process) and '\nOK\n' in process and 'FAILED' not in process
ids=set()
for text in [rust,qt,process]:
 for n in re.findall(r'vs2_(\d{3})_',text):ids.add(n)
# VS2-023 is an external source-bound review gate, never a self-authored unit pass.
reviewdir=ROOT/'docs/implementation/vslice-002/evidence'
a=json.loads((reviewdir/'review-qt.json').read_text());b=json.loads((reviewdir/'review-b.json').read_text())
for scope,review in [('qt',a),('b',b)]:
 assert review['status']=='APPROVED' and review['independentTests']
 for path,digest in review['sourceSha256'].items():assert sha(ROOT/path)==digest,path
 for path in review['sourceSha256']:
  destination=OUT/'sources'/path;destination.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/path,destination)
 shutil.copyfile(reviewdir/f'review-{scope}.json',OUT/f'review-{scope}.json')
assert a['reviewer']!=b['reviewer'];ids.add('023');assert ids=={f'{n:03d}' for n in range(1,25)},ids
reports=[]
for path in sorted((OUT/'process').glob('*/run-*/report.json')):
 r=json.loads(path.read_text());assert r['exitCode']==0 and not r['failure'] and r['guiThread'];assert r['pid']!=r['workerProcessPid']==r['finalState']['pid'];reports.append((path,r))
assert len(reports)==25,len(reports)
pids=[r['pid'] for _,r in reports];assert len(set(pids))==25
binary=ROOT/'target/desktop-build/workbench-desktop';digest=sha(binary)
assert all(r['binarySha256']==digest for _,r in reports)
cold=[r for p,r in reports if 'test_vs2_018_' in str(p)];assert len(cold)==2
assert cold[0]['pid']!=cold[1]['pid'] and cold[0]['workerProcessPid']!=cold[1]['workerProcessPid']
assert cold[1]['finalState']['invocations']=='0' and cold[1]['finalState']['projection']['values']['payment']=='1461.48'
assert cold[0]['finalState']['rooms']==cold[1]['finalState']['rooms']
offline=[p for p,_ in reports if 'test_vs2_022_' in str(p) and p.parent.name=='run-2'];assert len(offline)==1
assert 'OFFLINE_IPV4_IPV6_DENIED' in (offline[0].parent/'stdout.txt').read_text()
shutil.copyfile(binary,OUT/'workbench-desktop')
for folder in ['fixtures/vslice-002','docs/implementation/vslice-002']:
 dest=OUT/'contracts'/folder
 if folder.startswith('docs'):
  dest.mkdir(parents=True,exist_ok=True)
  for name in ['SPECIFICATION.md','TEST_MATRIX.md','VALIDATION_TASK.md','REVIEW_CHECKLISTS.md','IMPLEMENTATION_MAP.md']:
   shutil.copyfile(ROOT/folder/name,dest/name)
 else:shutil.copytree(ROOT/folder,dest,dirs_exist_ok=True)
summary={'headSha':run['headSha'],'trackedChanges':run['trackedChanges'],'cases':{f'VS2-{n:03d}':'executed; external hosted decision verified separately' for n in range(1,25)},'rustTestsPassed':sum(map(int,re.findall(r'test result: ok\. (\d+) passed',rust))),'qtSummary':re.search(r'Totals:.*',qt)[0],'processTests':14,'desktopProcesses':25,'workerProcesses':25,'appSha256':digest,'coldProcessIds':[r['pid'] for r in cold],'coldWorkerIds':[r['workerProcessPid'] for r in cold],'coldRestoreInvocations':0,'hostedExecution':os.environ.get('GITHUB_ACTIONS')=='true','reviewers':[a['reviewer'],b['reviewer']],'acceptance':'PROPOSED until hosted success and downloaded archive verification','limitations':['Linux single-writer clean-quit snapshots, not full database/power-loss recovery','one exact UI component instance per room; generic accepted form vocabulary','offline probe denies ordinary IPv4/IPv6 socket syscalls, not a general adversarial sandbox','offscreen native-window shell and canvas screenshots are separate supplemental captures']}
(OUT/'report.json').write_text(json.dumps(summary,indent=2)+'\n')
for p in sorted(OUT.rglob('*')):
 if p.is_file():run['sha256'][str(p.relative_to(REPORT))]=sha(p)
run['vs2Report']='vs2/report.json';(REPORT/'run.json').write_text(json.dumps(run,indent=2)+'\n')
print('PASS VS2-001..024 executed, real25-process boundaries, source-bound reviews and archive inventory; hosted acceptance remains external')
