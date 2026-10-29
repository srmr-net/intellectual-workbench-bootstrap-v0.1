"""Negative tests for hosted evidence identity, without network access."""
import copy
import unittest
from verify import hosted

class HostedIdentity(unittest.TestCase):
    def setUp(self):
        self.source = 'a' * 40
        self.head = 'b' * 40
        self.api = dict(status='completed', conclusion='success', event='pull_request', head_sha=self.head,
                        path='.github/workflows/ux-001.yml', id=123, run_attempt=2,
                        repository={'full_name': 'srmr-net/intellectual-workbench-bootstrap-v0.1'},
                        pull_requests=[{'number': 14}])
        self.run = {'github': dict(GITHUB_RUN_ID='123', GITHUB_RUN_ATTEMPT='2',
                    GITHUB_REPOSITORY=self.api['repository']['full_name'], GITHUB_SHA=self.source,
                    RUNNER_OS='Linux', GITHUB_REF='refs/pull/14/merge')}
        self.report = dict(runId='123', runAttempt='2', hostedExecution=True)
    def test_matching(self):
        hosted(self.api, self.run, self.report, self.source, self.head)
    def test_mismatched_api(self):
        for field, value in [('conclusion','failure'), ('event','workflow_dispatch'),
                             ('head_sha','c'*40), ('path','.github/workflows/vslice-004.yml'),
                             ('id',124), ('run_attempt',3)]:
            with self.subTest(field=field):
                api = copy.deepcopy(self.api); api[field] = value
                with self.assertRaises(AssertionError):
                    hosted(api, self.run, self.report, self.source, self.head)
    def test_mismatched_runner(self):
        for field, value in [('GITHUB_SHA','c'*40), ('RUNNER_OS','Windows'),
                             ('GITHUB_REF','refs/heads/master'), ('GITHUB_REPOSITORY','other/repo')]:
            with self.subTest(field=field):
                run = copy.deepcopy(self.run); run['github'][field] = value
                with self.assertRaises(AssertionError):
                    hosted(self.api, run, self.report, self.source, self.head)
if __name__ == '__main__':
    unittest.main()
