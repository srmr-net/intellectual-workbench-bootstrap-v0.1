"""Collect executed OP gates. This is never authority to self-accept an ADR."""
from pathlib import Path
import hashlib,json,os,re,shutil,subprocess
from contract import ROOT,RUST_TESTS,WORKER_TESTS,GUI_TESTS,NATIVE,CASES,critical_sources
REPORT=ROOT/'target/abi-evidence';OUT=Path(os.environ.get('WB_OPERATOR_EVIDENCE_DIR',REPORT/'operator'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rust=(REPORT/'workspace-tests.txt').read_text();worker=(OUT/'worker-tests.txt').read_text();gui=(OUT/'process-tests.txt').read_text()
assert not re.search(r'test \S+ \.\.\. (?:FAILED|ignored)',rust)
assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',rust)))==RUST_TESTS
for name in NATIVE.values():assert re.search(r'test (?:\w+::)*'+re.escape(name)+r' \.\.\. ok\b',rust),name
for text,count in [(worker,WORKER_TESTS),(gui,GUI_TESTS)]:assert f'Ran {count} tests' in text and '\nOK\n' in text and 'FAILED' not in text and 'skipped' not in text
source=critical_sources();reviews=[]
for name in ['agent-security','runtime-integration']:
 review_path=ROOT/f'docs/implementation/adr-005/evidence/review-{name}.json';review=json.loads(review_path.read_text())
 assert review['status']=='APPROVED' and review['independentTests'] and not review['remainingMandatory']
 assert review['reviewer']!='/root'
 subprocess.run(['git','merge-base','--is-ancestor',review['implementationCommit'],'HEAD'],cwd=ROOT,check=True)
 for path in source:assert review['sourceSha256'].get(path)==sha(ROOT/path),(name,path)
 shutil.copyfile(review_path,OUT/f'review-{name}.json')
 reviews.append(review)
assert reviews[0]['reviewer']!=reviews[1]['reviewer']
for path in source:
 dest=OUT/'sources'/path;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/path,dest)
shutil.copyfile(ROOT/'target/operator-build/workbench-operator',OUT/'workbench-operator')
reports=[(p,json.loads(p.read_text())) for p in (OUT/'process').rglob('report.json')]
assert len(reports)==7
assert len({r['pid'] for _,r in reports})==len({r['workerProcessPid'] for _,r in reports})==7
app_sha=sha(OUT/'workbench-operator')
for p,r in reports:assert r['exitCode']==0 and not r['failure'] and r['guiThread'] and r['binarySha256']==app_sha and all(s['operatorGuiThread'] for s in r['samples']),str(p)
def case(part):return next(r for p,r in reports if part in str(p))
happy=case('test_op_013_014_017_019_');samples=happy['samples'];preview=next(s for s in samples if s['action']=='intent');approval=next(s for s in samples if s['action']=='approve')
assert preview['state']['rooms']==[] and not preview['consentChecked'] and not preview['approveEnabled']
assert approval['operatorReply']['topologyCommitted'] and approval['operatorReply']['initializationSucceeded'] and approval['operatorReply']['checkpointPersisted']
assert approval['operatorReply']['initialInvocations']=='1'
assert next(s for s in samples if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='1'
assert happy['finalState']['projection']['values']['payment']=='1753.77'
cold=[(p,r) for p,r in reports if 'test_op_020_' in str(p)];assert len(cold)==2
cold.sort(key=lambda pair:str(pair[0]));first,last=[r for _,r in cold]
assert first['pid']!=last['pid'] and last['finalState']['invocations']=='0'
assert first['finalState']['rooms']==last['finalState']['rooms']
assert next(s for s in last['samples'] if s['action']=='operatorStatus')['operatorReply']['providerCalls']=='0'
assert last['finalState']['projection']['values']['payment']=='1461.48'
assert 'OFFLINE_IPV4_IPV6_DENIED' in cold[1][0].with_name('stdout.txt').read_text()
partial=case('test_op_018_');a=next(s for s in partial['samples'] if s['action']=='approve');assert a['operatorReply']['topologyCommitted'] and a['operatorReply']['topologyCheckpointPersisted'] and not a['operatorReply']['initializationSucceeded'];assert partial['finalState']['projection']['values']['payment']=='123.45'
run=json.loads((REPORT/'run.json').read_text())
summary={'headSha':run['headSha'],'trackedChanges':run['trackedChanges'],'cases':{f'OP-{n:03d}':{'nativeTests':[NATIVE[k] for k in CASES[n]],'processEvidence':'worker-tests.txt/process-tests.txt','status':'executed; external hosted/review verification required'} for n in range(1,25)},'rustTestsPassed':RUST_TESTS,'newNativeTests':len(NATIVE),'workerTests':WORKER_TESTS,'qtProcessTests':GUI_TESTS,'desktopProcesses':7,'workerProcesses':7,'providerCallsAfterDirectEdit':1,'coldProviderCalls':0,'coldGuestInvocations':0,'reviewers':[r['reviewer'] for r in reviews],'appSha256':app_sha,'workerSha256':sha(ROOT/'target/debug/wb-operator-worker'),'hostedExecution':os.environ.get('GITHUB_ACTIONS')=='true','acceptance':'PROPOSED until final successful hosted run and downloaded archive verification','limitations':['one installed component in an empty existing room; initial empty workspace UI profile','explicit MortgageTerms/GBP adapter and deterministic fixture, not general NL understanding','no external provider adapter, auto-approval or schema-repair retry','single-writer accepted snapshot1 handoff; database/power-loss recovery remains deferred','approval is commit boundary; later cancel/revoke serializes after apply']}
summary['sha256']={str(p.relative_to(OUT)):sha(p) for p in sorted(OUT.rglob('*')) if p.is_file() and p.name!='report.json'}
(OUT/'report.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='sha256'},indent=2))
