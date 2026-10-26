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
        assert report['rustTestsPassed'] == 99
        if args.hosted:
            assert report['hostedExecution'] and run['github']['RUNNER_OS'] == 'Linux'
            assert run['github']['GITHUB_SHA'] == args.source
        logs = archive.read('workspace-tests.txt').decode()
        assert sum(map(int, re.findall(r'test result: ok\. (\d+) passed', logs))) == 99
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
        print(json.dumps({'verified': True, 'source': args.source,
                          'recordedHashes': len(run['sha256']), 'rustTests': 99,
                          'numericalCases': 33, 'pin': receipt['pin']}, indent=2))


if __name__ == '__main__':
    main()
