"""Collect executed local/hosted gates, never infer final acceptance from CI YAML."""
import hashlib,json,os,re,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
REPORT=ROOT/'target/abi-evidence';UI=REPORT/'ui'
run=json.loads((REPORT/'run.json').read_text())
rust=(REPORT/'workspace-tests.txt').read_text();qt=(UI/'qt-tests.txt').read_text()
assert not re.search(r'FAIL!|SKIP\s+:|[1-9][0-9]* failed|[1-9][0-9]* skipped',qt),qt
required=['ui_009_011_', 'ui_012_014_', 'ui_015_016_017_', 'ui_009_010_017_', 'ui_018_']
for name in required:assert re.search(r'PASS\s+: UiConformance::'+name,qt),name
ids=set()
for match in re.findall(r'test (ui_[0-9_]+)[a-zA-Z]\S* \.\.\. ok',rust):
    ids.update(re.findall(r'\d{3}',match))
for match in re.findall(r'PASS\s+: UiConformance::(ui_[0-9_]+)[a-zA-Z]',qt):
    ids.update(re.findall(r'\d{3}',match))
assert ids=={f'{n:03d}' for n in range(1,21)},ids
reviews=[]
for name in ['qt','state']:
    path=ROOT/f'docs/implementation/adr-004/evidence/review-{name}.json'
    review=json.loads(path.read_text());assert review['status']=='APPROVED' and review['independentTests']
    for source,digest in review['sourceSha256'].items():assert hashlib.sha256((ROOT/source).read_bytes()).hexdigest()==digest,source
    for suffix in ['md','json']:shutil.copyfile(path.with_suffix('.'+suffix),UI/f'review-{name}.{suffix}')
    reviews.append(review['reviewer'])
assert len(set(reviews))==2
for name in ['mortgage-ui','alternate-ui']:
    shutil.copytree(ROOT/'target/ui-fixtures'/name,UI/'packages'/name,dirs_exist_ok=True)
receipt=json.loads((UI/'receipt.json').read_text());record=json.loads((UI/'record.json').read_text())
assert hashlib.sha256((UI/'record.json').read_bytes()).hexdigest()==receipt['record_digest']
assert record['checked']['ui']['sha256']==hashlib.sha256((UI/'packages/mortgage-ui/ui.json').read_bytes()).hexdigest()
assert record['pin']==receipt['pin']
for name,digest in record['files'].items():assert hashlib.sha256((UI/'packages/mortgage-ui'/name).read_bytes()).hexdigest()==digest,name
for path in ['form-dark-large.png','form-light.png','accessibility.json','projections.json','qt-tests.xml']:
    assert (UI/path).is_file() and (UI/path).stat().st_size>0,path
for source in ['schemas/ui.schema.json','apps/desktop-qt/qml/Main.qml','apps/desktop-qt/validate.sh','apps/desktop-qt/collect_evidence.py']:
    shutil.copyfile(ROOT/source,UI/Path(source).name)
summary={'headSha':run['headSha'],'trackedChanges':run['trackedChanges'],
 'cases':{f'UI-{n:03d}':'executed gates; review/hosted acceptance verified separately' for n in range(1,21)},
 'rustTestsPassed':sum(map(int,re.findall(r'test result: ok\. (\d+) passed',rust))),
 'qtSummary':re.search(r'Totals:.*',qt)[0], 'reviewers':reviews,
 'wasmSha256':record['pin']['sha256'],'uiSha256':record['checked']['ui']['sha256'],
 'schemaSha256':record['checked']['ui']['schema_sha256'],
 'hostedExecution':os.environ.get('GITHUB_ACTIONS')=='true',
 'acceptance':'PROPOSED until complete hosted run and downloaded archive verification',
 'limits':['Linux local directory profile; dynamic system Qt, offscreen software rendering',
 'one form, one aggregate input port, ui.json basename, direct scalar schema types',
 'u64 integer and revision/generation display transport is exact decimal text',
 'private adjacent snapshot fsync/rename, no SQLite or physical-power-cut guarantee',
 'no Qt guest imports, arbitrary package QML, Operator, remote registry or saved grants']}
(UI/'report.json').write_text(json.dumps(summary,indent=2)+'\n')
for p in sorted(UI.rglob('*')):
    if p.is_file():run['sha256'][str(p.relative_to(REPORT))]=hashlib.sha256(p.read_bytes()).hexdigest()
run['uiReport']='ui/report.json';(REPORT/'run.json').write_text(json.dumps(run,indent=2)+'\n')
print('PASS UI-001..020 executable, independent review and artifact gates; hosted decision remains external')
