"""VS4-029 negative provenance gates, without trusting a green unrelated run."""
import copy,importlib.util,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('live_verify',ROOT/'docs/implementation/vslice-004/evidence/verify.py');verify=importlib.util.module_from_spec(spec);spec.loader.exec_module(verify)
class Evidence(unittest.TestCase):
 def data(self):
  api={'status':'completed','conclusion':'success','event':'pull_request','head_sha':'head','path':'.github/workflows/vslice-004.yml','id':42,'run_attempt':1,'repository':{'full_name':'srmr-net/intellectual-workbench-bootstrap-v0.1'},'pull_requests':[{'number':11}]}
  run={'github':{'GITHUB_RUN_ID':'42','GITHUB_RUN_ATTEMPT':'1','GITHUB_REPOSITORY':api['repository']['full_name'],'GITHUB_SHA':'merge','RUNNER_OS':'Linux','GITHUB_REF':'refs/pull/11/merge'}}
  report={'runId':'42','runAttempt':'1','hostedExecution':True}
  return api,run,report
 def test_same_run_linux_pr_source_accepts(self):verify.validate_hosted_run(*self.data(),'merge','head')
 def test_wrong_run_attempt_workflow_or_source_rejected(self):
  expected={'a':'one','b':'two'}
  verify.validate_source_map(expected,expected)
  for actual in [{'a':'one'},{'a':'changed','b':'two'},{'a':'one','b':'two','extra':'three'}]:
   with self.assertRaises(AssertionError):verify.validate_source_map(actual,expected)
  for field,value in [('id',43),('run_attempt',2),('path','.github/workflows/vslice-003.yml'),('head_sha','other'),('conclusion','failure'),('status','in_progress'),('event','workflow_dispatch')]:
   api,run,report=self.data();api[field]=value
   with self.subTest(field=field),self.assertRaises(AssertionError):verify.validate_hosted_run(api,run,report,'merge','head')
  for field,value in [('GITHUB_SHA','other'),('GITHUB_REF','refs/heads/master'),('RUNNER_OS','Windows'),('GITHUB_REPOSITORY','evil/repo')]:
   api,run,report=self.data();run['github'][field]=value
   with self.subTest(field=field),self.assertRaises(AssertionError):verify.validate_hosted_run(api,run,report,'merge','head')
if __name__=='__main__':unittest.main(verbosity=2)
