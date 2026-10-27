"""Negative provenance tests: another successful workflow must never accept VS3."""
import copy,importlib.util,unittest
from pathlib import Path
root=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('vs3_verifier',root/'docs/implementation/vslice-003/evidence/verify.py');v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
class Provenance(unittest.TestCase):
 def setUp(self):
  self.source='a'*40;self.head='b'*40
  self.api={'id':42,'run_attempt':1,'path':'.github/workflows/vslice-003.yml','status':'completed','conclusion':'success','event':'pull_request','head_sha':self.head,'repository':{'full_name':'srmr-net/intellectual-workbench-bootstrap-v0.1'},'pull_requests':[{'number':10}]}
  self.run={'headSha':self.source,'github':{'GITHUB_RUN_ID':'42','GITHUB_RUN_ATTEMPT':'1','GITHUB_REPOSITORY':self.api['repository']['full_name'],'GITHUB_SHA':self.source,'RUNNER_OS':'Linux','GITHUB_REF':'refs/pull/10/merge'}};self.report={'source':self.source,'hostedExecution':True}
 def check(self):v.validate_hosted_run(self.api,self.run,self.report,self.source,self.head)
 def test_vs3_026_exact_run_binding(self):self.check()
 def test_vs3_026_other_successful_run_cannot_substitute(self):
  self.api['id']=43
  with self.assertRaises(AssertionError):self.check()
 def test_vs3_026_other_attempt_cannot_substitute(self):
  self.api['run_attempt']=2
  with self.assertRaises(AssertionError):self.check()
 def test_vs3_026_other_workflow_cannot_substitute(self):
  self.api['path']='.github/workflows/ci.yml'
  with self.assertRaises(AssertionError):self.check()
 def test_vs3_026_other_pull_request_cannot_substitute(self):
  self.run['github']['GITHUB_REF']='refs/pull/11/merge'
  with self.assertRaises(AssertionError):self.check()
 def test_vs3_026_failure_or_head_drift_cannot_substitute(self):
  for key,value in [('status','in_progress'),('conclusion','failure'),('head_sha','c'*40)]:
   old=self.api[key];self.api[key]=value
   with self.assertRaises(AssertionError):self.check()
   self.api[key]=old
if __name__=='__main__':unittest.main(verbosity=2)
