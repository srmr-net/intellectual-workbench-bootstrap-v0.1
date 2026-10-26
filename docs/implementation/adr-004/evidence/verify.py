#!/usr/bin/env python3
"""Offline evidence verification; hosted workflow conclusion is checked separately."""
import argparse
import hashlib
import json
import re
from pathlib import Path
from zipfile import ZipFile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--source', required=True)
    parser.add_argument('--hosted', action='store_true')
    args = parser.parse_args()
    assert hashlib.sha256(args.archive.read_bytes()).hexdigest() == args.sha256
    with ZipFile(args.archive) as archive:
        load = lambda name: json.loads(archive.read(name))
        run = load('run.json')
        assert run['headSha'] == args.source and not run['trackedChanges']
        assert all(v == 'passed' for v in run['checks'].values())
        for name, digest in run['sha256'].items():
            assert hashlib.sha256(archive.read(name)).hexdigest() == digest, name
        report = load('vslice/report.json')
        assert len(report['cases']) == 20 and report['numericalCases'] == 33
        assert len(report['regCases']) == 20 and len(report['ipcCases']) == 15
        assert len(report['abiCases']) == 10
        assert report['rustTestsPassed'] == 117
        if args.hosted:
            assert report['hostedExecution'] and run['github']['RUNNER_OS'] == 'Linux'
            assert run['github']['GITHUB_SHA'] == args.source
        logs = archive.read('workspace-tests.txt').decode()
        assert sum(map(int, re.findall(r'test result: ok\. (\d+) passed', logs))) == 117
        assert not re.search(r'test (?:vslice|reg|ipc|abi)_\S+ \.\.\. (?:FAILED|ignored)', logs)
        receipt = load('vslice/receipt.json')
        record = load('vslice/record.json')
        assert hashlib.sha256(archive.read('vslice/record.json')).hexdigest() == receipt['record_digest']
        assert record['pin'] == receipt['pin'] and record['checked']['trust'] == 'unreviewed'
        for name, digest in record['files'].items():
            assert hashlib.sha256(archive.read('vslice/packages/mortgage-headless/' + name)).hexdigest() == digest
        assert hashlib.sha256(archive.read('vslice/packages/mortgage-headless/logic.wasm')).hexdigest() == receipt['pin']['sha256']
        rebuild = load('vslice/rebuild.json')
        assert rebuild['firstSha256'] == rebuild['cleanSha256'] == receipt['pin']['sha256']
        results = load('vslice/numerical-results.json')
        assert len(results) == 33 and all(v['actual'] == v['expected'] for v in results)
        trace = load('vslice/trace.json')
        assert trace['pin'] == receipt['pin'] and trace['invocationId']
        assert trace['committedOutput'] == {'monthly_payment': '1461.48'}
        reviews = load('vslice/independent-reviews.json')
        assert reviews['status'] == 'APPROVED' and len(reviews['reviews']) == 2
        assert all(r['status'] == 'APPROVED' and r['independentTests'] for r in reviews['reviews'])
        ui = load('ui/report.json')
        assert set(ui['cases']) == {f'UI-{n:03d}' for n in range(1,21)}
        assert ui['headSha'] == args.source and not ui['trackedChanges']
        assert ui['rustTestsPassed'] == 117
        qt = archive.read('ui/qt-tests.txt').decode()
        assert '7 passed, 0 failed, 0 skipped' in qt
        assert not re.search(r'FAIL!|SKIP\s+:', qt)
        for case in ['ui_009_011_', 'ui_012_014_', 'ui_015_016_017_', 'ui_009_010_017_', 'ui_018_']:
            assert re.search(r'PASS\s+: UiConformance::'+case, qt), case
        if args.hosted: assert ui['hostedExecution']
        for scope in ['qt','state']:
            review = load('ui/review-'+scope+'.json')
            assert review['status'] == 'APPROVED' and review['independentTests']
            for path,digest in review['sourceSha256'].items():
                assert hashlib.sha256((Path(__file__).resolve().parents[4]/path).read_bytes()).hexdigest()==digest,path
        ui_receipt = load('ui/receipt.json'); ui_record = load('ui/record.json')
        assert hashlib.sha256(archive.read('ui/record.json')).hexdigest() == ui_receipt['record_digest']
        assert ui_record['pin'] == ui_receipt['pin']
        assert ui_record['pin']['id'] != receipt['pin']['id']
        assert ui_record['pin']['sha256'] == receipt['pin']['sha256']
        assert 'ui' not in record['checked']
        for name,digest in ui_record['files'].items():
            assert hashlib.sha256(archive.read('ui/packages/mortgage-ui/'+name)).hexdigest()==digest,name
        descriptor = ui_record['checked']['ui']
        assert descriptor['sha256'] == ui['uiSha256'] == hashlib.sha256(archive.read('ui/packages/mortgage-ui/ui.json')).hexdigest()
        assert descriptor['schema_sha256'] == ui['schemaSha256'] == hashlib.sha256(archive.read('ui/ui.schema.json')).hexdigest()
        for name in ['form-light.png','form-dark-large.png']:
            assert archive.read('ui/'+name).startswith(b'\x89PNG\r\n\x1a\n')
        accessibility=load('ui/accessibility.json'); assert len(accessibility)==3
        assert all(v['focusable'] and v['name'] for v in accessibility)
        assert 'Qt version' in archive.read('ui/qt-version.txt').decode()
        frozen = {
            'component.wit':'cf90e5114153e2737accd1fc7535e1e494e699ac68b4e99e8cc36c8270a204c0',
            'registry/component.schema.json':'15f2b7a2b84655e62f14fc521fe06a291fd00bf5fbcc120e509aaa84e9e1dd55',
            'registry/ontology.ttl':'1741a428b7ca8bd9d4a761659c30bc5f914fcac60d201ad5361cc54ca9cc3bda',
            'abi-tests.rs':'263f74a47a8daf26197f8b3e947fc51e73e05da4030ac008ea416ed4d9542c95',
            'registry/ipc-tests.rs':'52a67d69f36f85303816ba12255f3e9c9e06a62f20e1523ee29844e778de8238',
            'registry/registry-tests.rs':'1adcc72c4b320f2c824902f936244c7082085dd862ceac3882c75a7ce0490298',
            'vslice/vslice-tests.rs':'2ba5fdc85db824a530bc6816bde8ec42521543b059e9027f5c725ed0be6ad377',
        }
        for name,digest in frozen.items():assert hashlib.sha256(archive.read(name)).hexdigest()==digest,name
        print(json.dumps({'verified': True, 'source': args.source,
                          'recordedHashes': len(run['sha256']), 'rustTests': 117,
                          'numericalCases': 33, 'pin': receipt['pin'], 'uiPin':ui_receipt['pin'],
                          'uiCases':20, 'qtSummary':ui['qtSummary'], 'qtVersions':archive.read('ui/qt-version.txt').decode(),
                          'archiveSha256':args.sha256}, indent=2))


if __name__ == '__main__':
    main()
