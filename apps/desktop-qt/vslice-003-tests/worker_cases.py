"""Native product-entry tests: real admitted WASM and durable workspace handoff."""
from pathlib import Path
import json,os,resource,subprocess,sys,tempfile,unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'operator-tests'))
import worker_cases as accepted
ROOT=accepted.ROOT
accepted.WORKER=ROOT/'target/debug/wb-vslice003-worker'
accepted.EVIDENCE=Path(os.environ.get('WB_VS3_EVIDENCE_DIR',ROOT/'target/abi-evidence/vs3'))/'worker'
INTENT=accepted.INTENT
class Handoff(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.tmp=tempfile.TemporaryDirectory(prefix='vs3-worker-');cls.root=Path(cls.tmp.name);cls.registry=cls.root/'registry'
  subprocess.run([str(ROOT/'target/debug/wb-desktop-install'),str(cls.registry),str(accepted.PYTHON),str(ROOT/'tools/registry_validate.py'),str(ROOT/'target/debug/wb-registry-inspect'),str(ROOT/'target/ui-fixtures/mortgage-ui')],check=True,capture_output=True,timeout=90)
 @classmethod
 def tearDownClass(cls):cls.tmp.cleanup()
 def setUp(self):self.tmpcase=tempfile.TemporaryDirectory();self.workspace=Path(self.tmpcase.name)/'workspace';self.c=accepted.Client(self.registry,self.workspace,ignore_file_signal=True)
 def tearDown(self):self.c.kill();self.tmpcase.cleanup()
 def assemble(self):
  p=self.c.op({'type':'prepare','utterance':INTENT})['preview'];r=self.c.op({'type':'approve','digest':p['proposalDigest']});self.assertTrue(r['ok'],r);self.c.desktop({'type':'select','roomId':r['roomId']});return r
 def saved(self):return json.loads((self.workspace/'workspace.json').read_text())
 def test_vs3_021_append_preserves_full_pinned_checkpoint(self):
  self.assemble();self.c.desktop({'type':'pin','pinned':True});first=self.saved()['rooms'][0];r=self.assemble();saved=self.saved();self.assertEqual(saved['rooms'][0],first);self.assertNotEqual(saved['rooms'][1]['roomId'],first['roomId']);self.assertEqual(r['providerCalls'],'2');self.assertEqual(r['initialInvocations'],'1');self.c.close();self.c.record(self._testMethodName)
 def test_vs3_017_second_topology_write_failure_preserves_first(self):
  self.assemble();old=(self.workspace/'workspace.json').read_bytes();p=self.c.op({'type':'prepare','utterance':INTENT})['preview'];limit=resource.prlimit(self.c.p.pid,resource.RLIMIT_FSIZE);resource.prlimit(self.c.p.pid,resource.RLIMIT_FSIZE,(0,limit[1]))
  try:r=self.c.op({'type':'approve','digest':p['proposalDigest']})
  finally:resource.prlimit(self.c.p.pid,resource.RLIMIT_FSIZE,limit)
  self.assertFalse(r['ok']);self.assertTrue(r['topologyCommitted']);self.assertFalse(r['topologyCheckpointPersisted']);self.assertFalse(r['initializationSucceeded']);self.assertFalse(r['checkpointPersisted']);self.assertEqual(r['initialInvocations'],'0');self.assertEqual((self.workspace/'workspace.json').read_bytes(),old);self.c.close();self.c.record(self._testMethodName)
 def test_vs3_014_workspace_drift_then_fresh_preview(self):
  self.assemble();p=self.c.op({'type':'prepare','utterance':INTENT})['preview'];self.c.desktop({'type':'pin','pinned':True});old=(self.workspace/'workspace.json').read_bytes();r=self.c.op({'type':'approve','digest':p['proposalDigest']});self.assertEqual(r['code'],'STATE_CONFLICT');self.assertEqual((self.workspace/'workspace.json').read_bytes(),old);self.assemble();self.assertEqual(len(self.saved()['rooms']),2);self.assertEqual(self.c.op({'type':'approve','digest':p['proposalDigest']})['code'],'APPROVAL_REQUIRED');self.c.close();self.c.record(self._testMethodName)
 def test_vs3_024_cancel_queued_second_intent_no_persistence(self):
  self.assemble();old=(self.workspace/'workspace.json').read_bytes();self.c.send({'protocol':'operator.1','token':'prepare-second','command':{'type':'prepare','utterance':INTENT}});self.c.send({'protocol':'operator.1','token':'cancel-second','command':{'type':'cancel'}});replies=[self.c.read(True),self.c.read(True)];self.assertEqual({r['code'] for r in replies},{'CANCELLED'});self.assertEqual((self.workspace/'workspace.json').read_bytes(),old);self.c.close();self.c.record(self._testMethodName)
 def test_vs3_020_old_and_future_view_counters_fail_closed(self):
  self.assemble();old_state=self.c.state;p=old_state['projection'];stale={'protocol':'0.1','localToken':'stale','key':p['key'],'expectedRoomRevision':p['roomRevision'],'expectedGeneration':p['executionGeneration'],'command':{'type':'edit','controlId':'principal','proposedValue':'300000'}}
  proposal=self.c.op({'type':'prepare','utterance':INTENT})['preview'];reply=self.c.op({'type':'approve','digest':proposal['proposalDigest']});self.assertTrue(reply['ok']);before=(self.workspace/'workspace.json').read_bytes()
  rejected=self.c.desktop({'type':'ui','request':stale});self.assertFalse(rejected['accepted']);self.assertEqual(rejected['diagnostics'][0]['code'],'STALE_VIEW');self.assertEqual((self.workspace/'workspace.json').read_bytes(),before);self.assertEqual(rejected['invocations'],'0')
  selected=self.c.desktop({'type':'select','roomId':reply['roomId']});self.assertTrue(selected['accepted']);self.assertGreater(int(selected['epoch']),int(old_state['epoch']))
  p=selected['projection'];request={'protocol':'0.1','localToken':'future','key':p['key'],'expectedRoomRevision':p['roomRevision'],'expectedGeneration':str(2**64-1),'command':{'type':'edit','controlId':'principal','proposedValue':'300000'}}
  rejected=self.c.desktop({'type':'ui','request':request});self.assertFalse(rejected['accepted']);self.assertEqual(rejected['invocations'],'0');self.assertEqual(len(self.saved()['rooms']),2);self.assertEqual(self.saved()['rooms'][1]['outputs'],{'payment':{'monthly_payment':'1461.48'}})
  self.c.close();self.c.record(self._testMethodName)
if __name__=='__main__':unittest.main(verbosity=2)
