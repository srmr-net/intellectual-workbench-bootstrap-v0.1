"""Genuine Qt/native/registry/WASM mock-provider journey. No external network/key."""
import importlib.util,json,os,shutil,subprocess,hashlib
from pathlib import Path
import unittest
ROOT=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('accepted_process',ROOT/'apps/desktop-qt/vslice-003-tests/process_cases.py');base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
base.APP=ROOT/'target/llm-qt-build/workbench-live-provider'
base.EVIDENCE=Path(os.environ.get('WB_LLM_EVIDENCE_DIR',ROOT/'target/abi-evidence/llm'))
# Reuse setup/launch assertions only, without inheriting/rerunning accepted test cases.
class Journey(unittest.TestCase):
 setUpClass=classmethod(base.Journey.setUpClass.__func__)
 tearDownClass=classmethod(base.Journey.tearDownClass.__func__)
 setUp=base.Journey.setUp
 tearDown=base.Journey.tearDown
 snapshot=base.Journey.snapshot
 native=base.Journey.native
 mode='success'
 def launch(self,actions,offline=False,bad_config=False):
  # Accepted launcher is copied at runtime only to choose the additive worker.
  old_popen=subprocess.Popen
  def popen(command,**kw):
   command=[str(ROOT/'target/debug/wb-live-worker') if x==str(ROOT/'target/debug/wb-vslice003-worker') else x for x in command]
   env=kw['env'];env['WB_LLM_TEST_TRANSPORT']=self.mode;env.pop('WB_LLM_SECRET_FILE',None);env.pop('WB_LLM_BUDGET',None)
   if bad_config:env['WB_LLM_BUDGET']='invalid'
   return old_popen(command,**kw)
  with unittest.mock.patch.object(subprocess,'Popen',popen):return base.Journey.launch(self,actions,offline)
 def remote(self):return [{'type':'selectProvider','remote':True},{'type':'draftIntent','text':base.INTENT},{'type':'remoteConsent','consent':True},{'type':'previewDraft'}]
 def test_llm_001_offline_and_invalid_optional_config(self):
  r=self.launch([{'type':'intent','text':base.INTENT},{'type':'approve'},base.expect(values={'payment':'1461.48'}),base.QUIT],bad_config=True)
  self.assertEqual(r['finalState']['projection']['values']['payment'],'1461.48');self.assertEqual(r['samples'][0]['providerState']['remoteAttempts'],'0')
 def test_llm_002_no_remote_consent(self):
  r=self.launch([{'type':'selectProvider','remote':True},{'type':'intent','text':base.INTENT},base.QUIT]);s=r['samples'][1];self.assertFalse(s['operatorReply']['ok']);self.assertEqual(s['providerState']['remoteAttempts'],'0');self.assertEqual(r['finalState']['rooms'],[])
 def test_llm_016_017_approval_and_drift(self):
  r=self.launch(self.remote()+[{'type':'decline'},base.QUIT]);s=r['samples'][3];self.assertTrue(s['operatorReply']['ok']);self.assertFalse(s['approveEnabled']);self.assertFalse(s['consentChecked']);self.assertEqual(s['state']['rooms'],[]);self.assertEqual(r['finalState']['rooms'],[])
  self.native('fresh_preview_identity_stale_revision_and_expiry','real_admission_preview_approval_ingress_and_replay','op_009_010_offered_keys_actual_fields_schema_and_pin_drift_no_fallback')
 def test_llm_018_019_real_execution_edits_and_cold_restore(self):
  first=self.launch(self.remote()+[{'type':'approve'},base.expect(values={'payment':'1461.48'}),base.input_('principal','300000'),base.expect(values={'payment':'1753.77'}),{'type':'pin','pinned':True},{'type':'operatorStatus'},base.QUIT]);s=next(s for s in first['samples'] if s['action']=='previewDraft');self.assertEqual(s['operatorReply']['providerAudit']['observation']['status'],'OK');self.assertEqual(first['samples'][-2]['providerState']['remoteAttempts'],'1')
  second=self.launch([base.expect(ready=True,values={'payment':'1753.77'},invocations='0'),{'type':'operatorStatus'},base.QUIT],offline=True,bad_config=True);self.assertNotEqual(first['pid'],second['pid']);self.assertNotEqual(first['workerProcessPid'],second['workerProcessPid']);self.assertEqual(second['samples'][1]['providerState']['remoteAttempts'],'0');self.assertEqual(second['samples'][1]['operatorReply']['providerCalls'],'0')
 def test_llm_010_011_012_errors_no_mutation(self):
  for mode,status in [('refused','REFUSED'),('incomplete','TRUNCATED'),('auth','AUTH_FAILED'),('tool','PROVIDER_ERROR'),('invalid','SCHEMA_MISMATCH'),('unavailable','UNAVAILABLE')]:
   with self.subTest(mode=mode):
    self.mode=mode;r=self.launch(self.remote()+[base.QUIT]);s=r['samples'][3];self.assertFalse(s['operatorReply']['ok']);self.assertEqual(s['operatorReply']['providerAudit']['observation']['status'],status);self.assertEqual(r['finalState']['rooms'],[])
 def test_llm_015_020_switch_pending_cancels(self):
  self.mode='slow';actions=self.remote();actions[-1]['settle']=False
  r=self.launch(actions+[{'type':'selectProvider','remote':False},{'type':'intent','text':base.INTENT},{'type':'approve'},base.expect(values={'payment':'1461.48'}),base.QUIT]);self.assertEqual(len(r['finalState']['rooms']),1);self.assertEqual(r['samples'][4]['providerState']['profile']['provider'],'offline-fixture')
 def test_llm_020_consent_consumed_and_edit_revokes(self):
  r=self.launch(self.remote()+[{'type':'decline'},{'type':'previewDraft'},base.QUIT]);s=r['samples'][5];self.assertFalse(s['operatorReply']['ok']);self.assertEqual(s['providerState']['remoteAttempts'],'1');self.assertEqual(r['finalState']['rooms'],[])
  r=self.launch([{'type':'selectProvider','remote':True},{'type':'draftIntent','text':base.INTENT},{'type':'remoteConsent','consent':True},{'type':'draftIntent','text':base.INTENT+' '},{'type':'previewDraft'},base.QUIT]);self.assertEqual(r['samples'][4]['providerState']['remoteAttempts'],'0');self.assertEqual(r['finalState']['rooms'],[])
if __name__=='__main__':
 import unittest.mock
 unittest.main(verbosity=2)
