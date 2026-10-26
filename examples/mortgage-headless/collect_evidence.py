"""Archive executed VSLICE gates; external hosted success is verified separately.

No acceptance/trust elevation is inferred from guest metadata or a green YAML file.
"""
from pathlib import Path
import hashlib,json,os,re,shutil
ROOT=Path(__file__).resolve().parents[2]
REPORT=ROOT/'target/abi-evidence'
run=json.loads((REPORT/'run.json').read_text())
logs=(REPORT/'workspace-tests.txt').read_text()
ids=sorted(set(re.findall(r'test vslice_(\d{3})_\S+ \.\.\. ok',logs)))
assert ids==[f'{n:03d}' for n in range(1,21)]
assert not re.search(r'test vslice_\S+ \.\.\. (?:FAILED|ignored)',logs)
assert all(v=='passed' for v in run['checks'].values())
reg=json.loads((REPORT/'registry/report.json').read_text());assert len(reg['cases'])==20
ipc=json.loads((REPORT/'ipc-report.json').read_text());assert len(ipc['cases'])==15 and len(ipc['abiCases'])==10
reviews=json.loads((ROOT/'docs/implementation/vslice-001/review/RESULTS.json').read_text())
assert reviews['status']=='APPROVED'
assert {r['scope'] for r in reviews['reviews']}=={'numerical/schema','security/integration'}
for review in reviews['reviews']:
    assert review['status']=='APPROVED' and review['independentTests']
    for path,digest in review['sourceSha256'].items():
        assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==digest,path
files={
 'mortgage-guest.rs':'examples/wasm-components/mortgage-headless/src/lib.rs',
 'adversary-guest.rs':'examples/wasm-components/mortgage-adversary/src/lib.rs',
 'observer-guest.rs':'examples/wasm-components/mortgage-observer/src/lib.rs',
 'input.schema.json':'examples/mortgage-headless/schemas/mortgage-input.schema.json',
 'output.schema.json':'examples/mortgage-headless/schemas/mortgage-output.schema.json',
 'headless-adapter.rs':'crates/wb-kernel/src/headless.rs',
 'headless-cli.rs':'crates/wb-kernel/src/bin/wb-mortgage-headless.rs',
 'vslice-tests.rs':'crates/wb-kernel/tests/vslice.rs',
 'vectors.json':'fixtures/mortgage-vectors.json',
 'oracle.json':'target/mortgage-reference.json',
 'rebuild.json':'target/mortgage-rebuild.json',
 'independent-reviews.json':'docs/implementation/vslice-001/review/RESULTS.json',
 'build.sh':'examples/mortgage-headless/build.sh',
 'validate.sh':'examples/mortgage-headless/validate.sh',
 'reference.py':'examples/mortgage-headless/reference.py',
 'package.py':'examples/mortgage-headless/package.py',
 'rebuild.py':'examples/mortgage-headless/rebuild.py',
 'collect_evidence.py':'examples/mortgage-headless/collect_evidence.py',
}
for name,source in files.items():shutil.copyfile(ROOT/source,REPORT/'vslice'/name)
for name in ['mortgage-headless','mortgage-adversary','mortgage-observer']:
    shutil.copytree(ROOT/'target/mortgage-packages'/name,REPORT/'vslice/packages'/name,dirs_exist_ok=True)
    shutil.copyfile(ROOT/'target/mortgage-packages'/f'{name}-files.json',REPORT/'vslice'/f'{name}-files.json')
receipt=json.loads((REPORT/'vslice/receipt.json').read_text())
record=json.loads((REPORT/'vslice/record.json').read_text())
assert hashlib.sha256((REPORT/'vslice/record.json').read_bytes()).hexdigest()==receipt['record_digest']
assert receipt['pin']==record['pin'] and record['checked']['trust']=='unreviewed'
package=REPORT/'vslice/packages/mortgage-headless'
for name,digest in record['files'].items():assert hashlib.sha256((package/name).read_bytes()).hexdigest()==digest,name
assert hashlib.sha256((package/'logic.wasm').read_bytes()).hexdigest()==receipt['pin']['sha256']
trace=json.loads((REPORT/'vslice/trace.json').read_text());assert trace['pin']==receipt['pin']
assert trace['committedOutput']=={'monthly_payment':'1461.48'} and trace['invocationId']
results=json.loads((REPORT/'vslice/numerical-results.json').read_text());assert len(results)==33
for result in results:assert result['actual']==result['expected'],result['id']
rebuild=json.loads((REPORT/'vslice/rebuild.json').read_text());assert rebuild['firstSha256']==rebuild['cleanSha256']==receipt['pin']['sha256']
calibration=json.loads((REPORT/'vslice/budget-calibration.json').read_text())
assert calibration['defaultFuel']==2_000_000 and calibration['productFuel']==20_000_000
assert calibration['memoryBytes']==8*1024*1024 and calibration['epochTicks']==100
summary={
 'headSha':run['headSha'],'trackedChanges':run['trackedChanges'],
 'baseAdr003Merge':'ae5a0090ad46afa967b5b925b8b8287c1e295a17',
 'cases':{f'VSLICE-{n}':'production tests passed; external evidence gates checked separately' for n in ids},
 'regCases':list(reg['cases']),'ipcCases':list(ipc['cases']),'abiCases':ipc['abiCases'],
 'rustTestsPassed':sum(map(int,re.findall(r'test result: ok\. (\d+) passed',logs))),
 'numericalCases':len(results),'policy':calibration,
 'reviewers':[r['reviewer'] for r in reviews['reviews']],
 'hostedExecution':os.environ.get('GITHUB_ACTIONS')=='true',
 'acceptance':'PROPOSED until complete hosted workflow success and downloaded archive verification',
 'limitations':['no Qt/Operator/WASI/native guest services/remote registry',
                'headless required UI reference inert; snapshots recompute derived output',
                'prior output value retained on failure; accepted diagnostic invalidation unchanged',
                'same-machine rebuild verified; cross-machine hash identity not assumed',
                'process SIGKILL tests are not physical power-cut proof'],
}
(REPORT/'vslice/report.json').write_text(json.dumps(summary,indent=2)+'\n')
for p in sorted((REPORT/'vslice').rglob('*')):
    if p.is_file():run['sha256'][str(p.relative_to(REPORT))]=hashlib.sha256(p.read_bytes()).hexdigest()
run['vsliceReport']='vslice/report.json'
(REPORT/'run.json').write_text(json.dumps(run,indent=2)+'\n')
print(f'PASS: VSLICE-001..020 executable gates, both independent reviews and {len(results)} numerical cases archived for {run["headSha"]}; hosted acceptance still requires external verification.')
