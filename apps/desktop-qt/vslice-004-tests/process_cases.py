"""VS4: real Qt gestures, real admitted guests, accepted adapter + mock HTTP only."""
import hashlib, importlib.util, json, os, shutil, subprocess, unittest
import unittest.mock
from pathlib import Path
import control_cases
from contract import assert_private_artifact
ROOT=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('vs3_process',ROOT/'apps/desktop-qt/vslice-003-tests/process_cases.py')
base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
base.APP=ROOT/'target/vs4-qt-build/workbench-vslice004'
base.EVIDENCE=Path(os.environ.get('WB_VS4_EVIDENCE_DIR',ROOT/'target/abi-evidence/vs4'))
class Journey(unittest.TestCase):
 setUpClass=classmethod(base.Journey.setUpClass.__func__)
 tearDownClass=classmethod(base.Journey.tearDownClass.__func__)
 tearDown=base.Journey.tearDown
 snapshot=base.Journey.snapshot
 native=base.Journey.native
 def setUp(self):
  base.Journey.setUp(self);self.mode='success';self.key=self.path/'private-key';self.key.write_text('-'.join(['vs4','private','test','token','not','real']));self.key.chmod(0o600);self.capture=self.path/'outgoing.jsonl';self.key_enabled=True
 def launch(self,actions,offline=False):
  original=subprocess.Popen
  def popen(command,**kw):
   command=[str(ROOT/'target/debug/wb-vslice004-worker') if x==str(ROOT/'target/debug/wb-vslice003-worker') else x for x in command]
   if getattr(self,'use_launcher',False):command=[str(ROOT/'apps/desktop-qt/vslice-004/run.sh'),'--profile',str(self.path/'launcher-profile'),'--validation-actions',command[-3],'--validation-report',command[-1]]
   env=kw['env']
   for name in list(env):
    if name.startswith(('OPENROUTER_','OPENAI_','WB_LLM_','WB_VS4_')):env.pop(name)
   env.update(WB_VS4_TEST_MODE=self.mode,WB_VS4_TEST_CAPTURE=str(self.capture))
   if self.key_enabled:env['WB_LLM_SECRET_FILE']=str(self.key)
   return original(command,**kw)
  with unittest.mock.patch.object(subprocess,'Popen',popen):return base.Journey.launch(self,actions,offline)
 def remote(self,text=base.INTENT,settle=True):
  return [{'type':'selectProvider','remote':True},{'type':'intentKeyboard','text':text},{'type':'remoteConsent','consent':True},{'type':'sendReviewed','settle':settle}]
 def assemble(self):return self.remote()+[{'type':'approveKeyboard'},base.expect(values={'payment':'1461.48'})]
 def sample(self,r,action):return next(s for s in r['samples'] if s['action']==action)
 def no_mutation(self,r):self.assertEqual(r['finalState']['rooms'],[]);self.assertEqual(r['finalState']['invocations'],'0')
 def test_vs4_001_fresh_installed_desktop(self):
  self.use_launcher=True;self.workspace=self.path/'launcher-profile/workspace'
  r=self.launch([base.QUIT]);self.no_mutation(r);self.assertTrue(r['finalState']['installed']);self.assertTrue(r['samples'][0]['windowVisible'])
 def test_vs4_002_offline_default_absent_key(self):
  self.key_enabled=False;r=self.launch([{'type':'selectProvider','remote':True},{'type':'intent','text':base.INTENT},base.QUIT]);s=self.sample(r,'intent');self.assertEqual(s['providerState']['remoteAttempts'],'0');self.assertFalse(s['providerState']['source']['available']);self.no_mutation(r)
 def test_vs4_003_private_source_control(self):
  control_cases.exercise(self,"source")
  self.key_enabled=False;r=self.launch([{'type':'credentialSource','path':str(self.key)},*self.remote(),{'type':'decline'},base.QUIT]);s=self.sample(r,'sendReviewed');self.assertTrue(s['operatorReply']['ok']);self.assertNotIn(self.key.read_text(),json.dumps(r));self.assertEqual(s['providerState']['source']['kind'],'private-file')
 def test_vs4_004_truthful_outbound_data(self):
  r=self.launch(self.remote()+[{'type':'decline'},base.QUIT]);s=self.sample(r,'intentKeyboard');d=s['disclosure'];self.assertEqual(d['outboundData']['utterance'],base.INTENT);self.assertEqual(len(d['outboundData']['candidates']),1);self.assertEqual(s['providerState']['remoteAttempts'],'0');self.assertIn('generated at send',s['outboundText']);self.assertNotIn(str(self.workspace),s['outboundText'])
  actual=json.loads(self.capture.read_text().splitlines()[0]);data=json.loads(actual['messages'][1]['content']);self.assertEqual(data['utterance'],d['outboundData']['utterance']);offered=data['candidates'][0];shown=d['outboundData']['candidates'][0];self.assertEqual({k:v for k,v in offered.items() if k!='candidateKey'},{k:v for k,v in shown.items() if k!='candidateKey'});self.assertRegex(offered['candidateKey'],r'^[0-9a-f]{64}$');self.assertNotIn(self.key.read_text(),self.capture.read_text())
 def test_vs4_005_withheld_denied_withdrawn(self):
  control_cases.exercise(self,"bypass")
  r=self.launch(self.remote()[:2]+[{'type':'remoteConsent','consent':True},{'type':'remoteConsent','consent':False},{'type':'draftIntent','text':base.INTENT+' '},base.QUIT]);self.no_mutation(r);self.assertFalse(self.capture.exists());self.assertFalse(r['samples'][-1]['remoteConsent'])
 def test_vs4_006_assistive_consent_state(self):
  r=self.launch(self.remote()+[{'type':'decline'},base.QUIT]);s=self.sample(r,'remoteConsent');w={v['id']:v for v in s['operatorWidgets']};self.assertTrue(w['remoteConsent']['focusable']);self.assertTrue(w['operatorSend']['enabled']);self.assertFalse(w['operatorApprove']['enabled']);self.assertTrue(w['outboundData']['name'])
 def test_vs4_007_actual_adapter_transport(self):
  r=self.launch(self.remote()+[{'type':'decline'},base.QUIT]);s=self.sample(r,'sendReviewed');self.assertEqual(s['providerState']['transport'],'injected-mock-http');self.assertEqual(s['providerState']['remoteAttempts'],'1');self.assertEqual(s['operatorReply']['providerAudit']['observation']['status'],'OK');self.assertTrue(self.capture.exists());self.no_mutation(r)
 def test_vs4_008_progress_cancel(self):
  self.mode='delayed';r=self.launch(self.remote(settle=False)+[{'type':'wait','milliseconds':150},{'type':'cancel'},{'type':'wait','milliseconds':1700},base.QUIT]);self.assertTrue(self.sample(r,'sendReviewed')['operatorBusy']);w={x['id']:x for x in self.sample(r,'wait')['operatorWidgets']};self.assertTrue(all(w[k]['inViewport'] for k in ['operatorStatus','operatorCancel','requestProgress']));self.assertFalse(r['samples'][-1]['hasPreview']);self.no_mutation(r)
 def test_vs4_009_exact_admitted_selection(self):
  r=self.launch(self.remote()+[{'type':'decline'},base.QUIT]);s=self.sample(r,'sendReviewed');p=s['operatorReply']['preview'];self.assertIn(p['plan']['instances'][0]['component'],[i['pin'] for i in s['state']['installed']]);self.assertEqual(p['plan']['planVersion'],'0.1');self.assertTrue(all(v for k,v in p['validation'].items() if k!='profile'))
 def test_vs4_010_adversarial_suggestions(self):
  for mode in ['candidate','port','duplicate','value']:
   with self.subTest(mode=mode):self.mode=mode;r=self.launch(self.remote()+[base.QUIT]);self.assertFalse(self.sample(r,'sendReviewed')['operatorReply']['ok']);self.no_mutation(r)
 def test_vs4_011_injection_no_authority(self):
  self.mode='injection';r=self.launch(self.remote()+[base.QUIT]);self.assertFalse(self.sample(r,'sendReviewed')['operatorReply']['ok']);self.no_mutation(r);self.native('op_002_003_008_discovery_is_admitted_only_minimal_readonly_and_instruction_free')
 def test_vs4_012_structured_preview(self):
  r=self.launch(self.remote()+[{'type':'decline'},base.QUIT]);s=self.sample(r,'sendReviewed');pin=s['operatorReply']['preview']['plan']['instances'][0]['component'];text=s['previewText']
  w={x['id']:x for x in s['operatorWidgets']};self.assertTrue(w['operatorConsent']['inViewport']);self.assertTrue(w['proposalDetails']['inViewport'])
  for value in [pin['id'],pin['version'],pin['sha256'],'GBP','months','Affected room','Requested permissions','separate operations','Proposal digest','Expiry']:self.assertIn(value,text)
 def test_vs4_013_separate_genuine_approval(self):
  r=self.launch(self.remote()+[{'type':'approveWithoutConsent'},{'type':'decline'},base.QUIT]);self.no_mutation(r);s=self.sample(r,'sendReviewed');self.assertFalse(s['approveEnabled']);self.assertFalse(s['consentChecked']);self.assertFalse(s['remoteConsent'])
 def test_vs4_014_reject_replay_double_click(self):
  control_cases.exercise(self,"replay")
  r=self.launch(self.remote()+[{'type':'decline'},*self.remote(),{'type':'doubleApprove'},base.expect(rooms=1,values={'payment':'1461.48'}),base.QUIT]);self.assertEqual(len(r['finalState']['rooms']),1);self.native('fresh_preview_identity_stale_revision_and_expiry','real_admission_preview_approval_ingress_and_replay')
  self.workspace=self.path/'expired-proposal';r=self.launch(self.remote()+[{'type':'wait','milliseconds':31000} for _ in range(4)]+[{'type':'approveWithoutConsent'},base.QUIT]);self.assertFalse(r['samples'][-1]['hasPreview']);self.no_mutation(r)
 def test_vs4_015_revision_pin_fences(self):
  control_cases.exercise(self,"durable")
  r=self.launch(self.assemble()+self.remote()+[{'type':'pin','pinned':True},{'type':'approve'},base.expect(rooms=1,values={'payment':'1461.48'}),base.QUIT]);approvals=[s['operatorReply'] for s in r['samples'] if s['action']=='approve'];self.assertEqual(approvals[-1]['code'],'STATE_CONFLICT');self.native('op_009_010_offered_keys_actual_fields_schema_and_pin_drift_no_fallback')
 def test_vs4_016_real_kernel_wasm(self):
  r=self.launch(self.assemble()+[base.QUIT]);a=self.sample(r,'approveKeyboard')['operatorReply'];self.assertTrue(a['topologyCommitted'] and a['topologyCheckpointPersisted'] and a['initializationSucceeded']);self.assertEqual(a['initialInvocations'],'1');self.assertTrue(any(e['kind']=='invocation.started' for e in a['nativeAudit']))
 def test_vs4_017_partial_failure_safe_form_repair(self):
  self.registry=self.path/'trap-registry';subprocess.run([str(base.NATIVE/'wb-desktop-install'),str(self.registry),str(base.PYTHON),str(ROOT/'tools/registry_validate.py'),str(base.NATIVE/'wb-registry-inspect'),str(ROOT/'target/vs2-fixtures/adversary-ui')],check=True,capture_output=True,timeout=90)
  r=self.launch(self.remote('I need a mortgage calculator for a £5.00 loan over 25 years at 5%.')+[{'type':'approve'},base.expect(rooms=1,values={'payment':''}),*base.calculate('1.00','5','300'),base.expect(values={'payment':'123.45'}),{'type':'operatorStatus'},base.QUIT]);s=self.sample(r,'approve');self.assertTrue(s['operatorReply']['topologyCommitted']);self.assertFalse(s['operatorReply']['initializationSucceeded']);self.assertIn('Edit the installed form',s['operatorStatus']);self.assertEqual(self.sample(r,'operatorStatus')['providerState']['remoteAttempts'],'1')
 def test_vs4_018_committed_reference_output(self):
  r=self.launch(self.assemble()+[base.QUIT]);self.assertEqual(r['finalState']['projection']['values']['payment'],'1461.48');self.assertTrue(any(c['id']=='payment' and c['text']=='1461.48' for s in r['samples'] for c in s.get('controls',[])))
 def test_vs4_019_reactive_without_inference(self):
  r=self.launch(self.assemble()+[base.input_('principal','300000'),base.expect(values={'payment':'1753.77'}),{'type':'operatorStatus'},base.QUIT]);self.assertEqual(self.sample(r,'operatorStatus')['providerState']['remoteAttempts'],'1');self.assertEqual(len(self.capture.read_text().splitlines()),1)
 def test_vs4_020_rapid_edits_and_rollback(self):
  r=self.launch(self.assemble()+[base.input_('principal','300000',settle=False),base.input_('principal','350000'),{'type':'applyIfConflict'},base.expect(values={'payment':'2046.07'}),base.input_('principal','bad'),base.expect(values={'payment':'2046.07'},error='SCHEMA VIOLATION'),{'type':'operatorStatus'},base.QUIT]);self.assertEqual(self.sample(r,'operatorStatus')['providerState']['remoteAttempts'],'1')
 def test_vs4_021_provider_errors_preserve_commit(self):
  self.launch(self.assemble()+[base.QUIT]);saved=self.snapshot()
  for mode in ['refused','rate','auth','server','network','timeout','malformed','truncated','tool']:
   with self.subTest(mode=mode):
    self.mode=mode;r=self.launch(self.remote()+[base.QUIT]);s=self.sample(r,'sendReviewed');self.assertFalse(s['operatorReply']['ok']);self.assertFalse(s['hasPreview']);self.assertEqual(self.snapshot(),saved);self.assertIn(':',s['operatorStatus'])
 def test_vs4_022_switch_fences_and_isolation(self):
  self.launch(self.assemble()+[{'type':'selectProvider','remote':False},{'type':'intent','text':'I need a mortgage calculator for a £300,000 loan over 25 years at 5%.'},{'type':'approve'},base.expect(rooms=2,values={'payment':'1753.77'}),base.QUIT]);rooms=json.loads(self.snapshot())['rooms'];self.mode='delayed';r=self.launch(self.remote(settle=False)+[{'type':'selectProvider','remote':False},{'type':'wait','milliseconds':1700},base.expect(rooms=2,values={'payment':'1753.77'}),base.QUIT]);self.assertFalse(r['samples'][-1]['hasPreview']);r=self.launch(self.remote(settle=False)+[{'type':'select','index':0},{'type':'wait','milliseconds':1700},base.expect(rooms=2,values={'payment':'1461.48'}),{'type':'select','index':1},base.expect(values={'payment':'1753.77'}),base.QUIT]);self.assertFalse(r['samples'][-1]['hasPreview']);self.assertEqual(json.loads(self.snapshot())['rooms'],rooms)
 def test_vs4_023_pinned_cold_process_offline(self):
  a=self.launch(self.assemble()+[base.input_('principal','300000'),base.expect(values={'payment':'1753.77'}),{'type':'pin','pinned':True},base.QUIT]);before=self.snapshot();b=self.launch([base.expect(ready=True,values={'payment':'1753.77'},invocations='0'),{'type':'operatorStatus'},base.QUIT],offline=True);self.assertNotEqual(a['pid'],b['pid']);self.assertNotEqual(a['workerProcessPid'],b['workerProcessPid']);self.assertEqual(a['finalState']['rooms'],b['finalState']['rooms']);self.assertEqual(before,self.snapshot());s=self.sample(b,'operatorStatus');self.assertEqual(s['operatorReply']['providerCalls'],'0');self.assertEqual(s['providerState']['remoteAttempts'],'0')
 def test_vs4_024_offline_after_live_disabled(self):
  r=self.launch(self.remote()+[{'type':'decline'},{'type':'selectProvider','remote':False},{'type':'intent','text':base.INTENT},{'type':'approve'},base.expect(values={'payment':'1461.48'}),base.input_('principal','300000'),base.expect(values={'payment':'1753.77'}),{'type':'operatorStatus'},base.QUIT]);self.assertEqual(self.sample(r,'operatorStatus')['providerState']['remoteAttempts'],'1')
 def test_vs4_025_keyboard_focus_accessible_status(self):
  r=self.launch(self.remote()+[{'type':'tabJourney'},{'type':'approveKeyboard'},base.expect(values={'payment':'1461.48'}),base.QUIT]);s=self.sample(r,'sendReviewed');self.assertGreater(s['statusAlerts'],0);names=self.sample(r,'focusTraversal')['names'];self.assertIn('providerChoice',names);self.assertIn('operatorConsent',names);self.assertIn('operatorApprove',names);self.assertTrue(all(s.get('operatorGuiThread',True) for s in r['samples']))
 def test_vs4_026_secret_safe_artifacts_and_negative_scan(self):
  self.mode='secret-error';r=self.launch(self.remote()+[base.QUIT]);secret=self.key.read_bytes();self.assertNotIn(secret,json.dumps(r).encode());self.assertNotIn(secret,self.capture.read_bytes());directory=base.EVIDENCE/'process'/self._testMethodName
  for p in directory.rglob('*'):
   if p.is_file():self.assertNotIn(secret,p.read_bytes(),str(p))
  with self.assertRaises(AssertionError):assert_private_artifact(b'negative scan fixture '+secret)
  assert_private_artifact(b'sanitized AUTH_FAILED');self.no_mutation(r)
if __name__=='__main__':unittest.main(verbosity=2)
