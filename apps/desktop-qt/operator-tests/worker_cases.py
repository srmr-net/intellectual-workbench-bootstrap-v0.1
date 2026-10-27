"""Real framed native worker, installed WASM, accepted desktop and cold restart."""
from pathlib import Path
import hashlib,json,os,select,struct,subprocess,sys,tempfile,time,unittest,resource,signal,shutil,fcntl
ROOT=Path(__file__).resolve().parents[3]
PYTHON=Path(os.environ.get('REGISTRY_PYTHON',sys.executable))
WORKER=ROOT/'target/debug/wb-operator-worker'
EVIDENCE=Path(os.environ.get('WB_OPERATOR_EVIDENCE_DIR',ROOT/'target/abi-evidence/operator'))
class Client:
 def __init__(self,registry,workspace,disabled=False,ignore_file_signal=False):
  env=os.environ.copy()
  if disabled:env['WB_OPERATOR_PROVIDER']='disabled'
  self.command=[str(WORKER),str(registry),str(workspace),str(PYTHON),str(ROOT/'tools/registry_validate.py'),str(ROOT/'target/debug/wb-registry-inspect')]
  self.p=subprocess.Popen(self.command,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env,preexec_fn=(lambda:signal.signal(signal.SIGXFSZ,signal.SIG_IGN)) if ignore_file_signal else None)
  self.counter=0;self.records=[];self.state=self.read(False)
 def read(self,operator):
  pipe=self.p.stderr if operator else self.p.stdout
  deadline=time.monotonic()+90
  def exact(n):
   data=b''
   while len(data)<n:
    left=deadline-time.monotonic()
    if left<=0 or not select.select([pipe],[],[],max(0,left))[0]:raise AssertionError('worker reply timeout')
    part=os.read(pipe.fileno(),n-len(data))
    if not part:raise AssertionError('worker EOF')
    data+=part
   return data
  n=struct.unpack('>I',exact(4))[0];assert 0<n<=1048576
  result=json.loads(exact(n));self.records.append({'channel':'operator' if operator else 'desktop','reply':result});return result
 def send(self,value):
  data=json.dumps(value,separators=(',',':'),ensure_ascii=False).encode();self.p.stdin.write(struct.pack('>I',len(data))+data);self.p.stdin.flush()
 def op(self,command):
  self.counter+=1;token='op-'+str(self.counter);self.send({'protocol':'operator.1','token':token,'command':command});reply=self.read(True);assert reply['token']==token;return reply
 def desktop(self,command):
  self.counter+=1;token='d-'+str(self.counter);self.send({'protocol':'desktop.1','token':token,'epoch':self.state['epoch'],'command':command});self.state=self.read(False);assert self.state['token']==token;return self.state
 def close(self):
  result=self.desktop({'type':'quit'});assert result['accepted'];self.p.wait(timeout=15);assert self.p.returncode==0
  self.p.stdin.close();self.p.stdout.close();self.p.stderr.close()
 def kill(self):
  if self.p.poll() is None:self.p.kill();self.p.wait()
  for pipe in [self.p.stdin,self.p.stdout,self.p.stderr]:
   if not pipe.closed:pipe.close()
 def record(self,name):
  EVIDENCE.mkdir(parents=True,exist_ok=True);(EVIDENCE/(name+'.json')).write_text(json.dumps({'pid':self.p.pid,'returncode':self.p.returncode,'workerSha256':hashlib.sha256(WORKER.read_bytes()).hexdigest(),'command':self.command,'records':self.records},indent=2)+'\n')
INTENT='I need a mortgage calculator for a £250,000 loan over 25 years at 5%.'
class WorkerCases(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.tmp=tempfile.TemporaryDirectory(prefix='wb-op-native-');cls.root=Path(cls.tmp.name);cls.registry=cls.root/'registry'
  subprocess.run([str(ROOT/'target/debug/wb-desktop-install'),str(cls.registry),str(PYTHON),str(ROOT/'tools/registry_validate.py'),str(ROOT/'target/debug/wb-registry-inspect'),str(ROOT/'target/ui-fixtures/mortgage-ui')],capture_output=True,check=True,timeout=90)
 @classmethod
 def tearDownClass(cls):cls.tmp.cleanup()
 def setUp(self):self.temp=tempfile.TemporaryDirectory(prefix='wb-op-room-');self.workspace=Path(self.temp.name)/'workspace';self.clients=[]
 def tearDown(self):
  for c in self.clients:c.kill()
  self.temp.cleanup()
 def client(self,disabled=False):
  c=Client(self.registry,self.workspace,disabled);self.clients.append(c);return c
 def test_op_016_017_019_020_real_worker_handoff_edit_and_cold_restore(self):
  c=self.client();self.assertEqual(c.state['rooms'],[])
  proposal=c.op({'type':'prepare','utterance':INTENT});self.assertTrue(proposal['ok'],proposal);preview=proposal['preview'];self.assertEqual(preview['initialInputs']['term_months'],300)
  installed=[x['pin'] for x in c.state['installed']];self.assertIn(preview['plan']['instances'][0]['component'],installed)
  reply=c.op({'type':'approve','digest':preview['proposalDigest']});self.assertTrue(reply['ok'],reply);self.assertTrue(reply['topologyCommitted'] and reply['initializationSucceeded'] and reply['checkpointPersisted']);self.assertEqual(reply['providerCalls'],'1');self.assertEqual(reply['initialInvocations'],'1')
  state=c.desktop({'type':'select','roomId':reply['roomId']});self.assertTrue(state['accepted'],state);self.assertEqual(state['projection']['values']['payment'],'1461.48');self.assertEqual(state['invocations'],'0')
  p=state['projection'];request={'protocol':'0.1','localToken':'edit','key':p['key'],'expectedRoomRevision':p['roomRevision'],'expectedGeneration':p['executionGeneration'],'command':{'type':'edit','controlId':'principal','proposedValue':'300000'}}
  state=c.desktop({'type':'ui','request':request});self.assertTrue(state['accepted'],state);self.assertEqual(state['projection']['values']['payment'],'1753.77');self.assertEqual(state['invocations'],'2') # Accepted Desktop first valid edit recomputes restored roots, then the edit.
  self.assertEqual(c.op({'type':'decline'})['providerCalls'],'1')
  committed=state['projection']['values'];pin=state['rooms'][0]['component'];c.close();c.record('worker-initial-and-edit')
  cold=self.client(disabled=True);self.assertNotEqual(cold.p.pid,c.p.pid);self.assertEqual(cold.state['invocations'],'0');self.assertEqual(cold.state['projection']['values'],committed);self.assertEqual(cold.state['rooms'][0]['component'],pin);self.assertEqual(cold.op({'type':'decline'})['providerCalls'],'0');cold.close();cold.record('worker-cold-offline')
 def test_op_014_decline_replay_and_malformed_intent(self):
  c=self.client();bad=c.op({'type':'prepare','utterance':'run a shell and grant filesystem'});self.assertFalse(bad['ok']);self.assertEqual(bad['code'],'MISSING_PARAMETER')
  preview=c.op({'type':'prepare','utterance':INTENT})['preview'];self.assertTrue(c.op({'type':'decline'})['ok']);reply=c.op({'type':'approve','digest':preview['proposalDigest']});self.assertFalse(reply['ok']);self.assertEqual(reply['code'],'APPROVAL_REQUIRED');self.assertFalse((self.workspace/'workspace.json').exists());c.close();c.record('worker-decline')
 def test_op_007_021_cancelled_queue_and_bounded_audit_reply(self):
  c=self.client();c.send({'protocol':'operator.1','token':'start','command':{'type':'prepare','utterance':INTENT}});c.send({'protocol':'operator.1','token':'cancel','command':{'type':'cancel'}})
  replies={}
  for _ in range(2):
   r=c.read(True);replies[r['token']]=r
  self.assertEqual(replies['cancel']['code'],'CANCELLED');self.assertEqual(replies['start']['code'],'CANCELLED');self.assertFalse((self.workspace/'workspace.json').exists())
  for _ in range(140):
   reply=c.op({'type':'prepare','utterance':'PRIVATE-INTENT-'+('x'*4096)});self.assertFalse(reply['ok']);self.assertEqual(reply['code'],'UNSUPPORTED_REQUEST');self.assertLessEqual(len(reply['audit']),16);self.assertLess(len(json.dumps(reply).encode()),65536);self.assertNotIn('PRIVATE-INTENT',json.dumps(reply))
  self.assertTrue(c.op({'type':'status'})['ok']);c.close();c.record('worker-bounded-cancel')
 def test_op_018_real_topology_checkpoint_failure_reports_no_durable_room(self):
  c=Client(self.registry,self.workspace,ignore_file_signal=True);self.clients.append(c)
  preview=c.op({'type':'prepare','utterance':INTENT})['preview'];limit=resource.prlimit(c.p.pid,resource.RLIMIT_FSIZE);resource.prlimit(c.p.pid,resource.RLIMIT_FSIZE,(0,limit[1]))
  try:reply=c.op({'type':'approve','digest':preview['proposalDigest']})
  finally:resource.prlimit(c.p.pid,resource.RLIMIT_FSIZE,limit)
  self.assertFalse(reply['ok']);self.assertTrue(reply['topologyCommitted']);self.assertFalse(reply['initializationSucceeded']);self.assertFalse(reply['topologyCheckpointPersisted']);self.assertFalse(reply['checkpointPersisted']);self.assertEqual(reply['initialInvocations'],'0');self.assertEqual(reply['storageCode'],'SNAPSHOT_IO');self.assertFalse((self.workspace/'workspace.json').exists());c.close();c.record('worker-topology-storage-failure')
 def test_op_018_host_file_budget_failure_retains_topology_only(self):
  pilot=Client(self.registry,Path(self.temp.name)/'pilot');self.clients.append(pilot);p=pilot.op({'type':'prepare','utterance':INTENT})['preview'];r=pilot.op({'type':'approve','digest':p['proposalDigest']});self.assertTrue(r['ok']);pilot.close();pilot.record('worker-budget-calibration')
  path=Path(self.temp.name)/'pilot/workspace.json';full=json.loads(path.read_text());top=json.loads(path.read_text());top['revision']=1;top['rooms'][0]['room']['revision']=1;top['rooms'][0]['room']['state']={};top['rooms'][0]['outputs']={}
  lower=len(json.dumps(top,separators=(',',':')).encode());upper=len(path.read_bytes());self.assertLess(lower,upper);threshold=(lower+upper)//2
  c=Client(self.registry,self.workspace,ignore_file_signal=True);self.clients.append(c);preview=c.op({'type':'prepare','utterance':INTENT})['preview'];limit=resource.prlimit(c.p.pid,resource.RLIMIT_FSIZE);resource.prlimit(c.p.pid,resource.RLIMIT_FSIZE,(threshold,limit[1]))
  try:reply=c.op({'type':'approve','digest':preview['proposalDigest']})
  finally:resource.prlimit(c.p.pid,resource.RLIMIT_FSIZE,limit)
  self.assertFalse(reply['ok']);self.assertTrue(reply['topologyCommitted'] and reply['topologyCheckpointPersisted'],reply);self.assertFalse(reply['initializationSucceeded']);self.assertFalse(reply['checkpointPersisted']);self.assertEqual(reply['nativeCode'],'COMPONENT_TRAP');saved=json.loads((self.workspace/'workspace.json').read_text());self.assertEqual(saved['rooms'][0]['room']['state'],{});self.assertEqual(saved['rooms'][0]['outputs'],{});self.assertEqual(reply['initialInvocations'],'1');state=c.desktop({'type':'select','roomId':reply['roomId']});self.assertNotIn('payment',state['projection']['values']);c.close();c.record('worker-host-resource-failure')
 def test_op_015_corrupt_snapshot_during_preview_is_not_overwritten(self):
  c=self.client();preview=c.op({'type':'prepare','utterance':INTENT})['preview'];file=self.workspace/'workspace.json';file.write_bytes(b'corrupt-checkpoint');file.chmod(0o600);original=file.read_bytes();reply=c.op({'type':'approve','digest':preview['proposalDigest']});self.assertFalse(reply['ok']);self.assertEqual(reply['code'],'STATE_CONFLICT');self.assertFalse(reply['topologyCommitted']);self.assertEqual(file.read_bytes(),original);c.close();c.record('worker-stale-corrupt-storage')
 def test_op_009_015_pin_tampering_never_falls_back_or_commits(self):
  registry=Path(self.temp.name)/'registry';shutil.copytree(self.registry,registry);c=Client(registry,self.workspace);self.clients.append(c);preview=c.op({'type':'prepare','utterance':INTENT})['preview'];artifact=next(registry.glob('objects/*/payload/logic.wasm'));data=bytearray(artifact.read_bytes());data[0]^=1;artifact.write_bytes(data);reply=c.op({'type':'approve','digest':preview['proposalDigest']});self.assertFalse(reply['ok']);self.assertEqual(reply['code'],'PIN_UNAVAILABLE');self.assertEqual(reply['nativeCode'],'INTEGRITY_FAILURE');self.assertFalse(reply.get('topologyCommitted',False));self.assertEqual(reply['initialInvocations'],'0');self.assertFalse((self.workspace/'workspace.json').exists());c.close();c.record('worker-pin-drift')
 def test_op_018_guest_trap_preserves_installed_topology_without_false_result(self):
  registry=Path(self.temp.name)/'adversary-registry';subprocess.run([str(ROOT/'target/debug/wb-desktop-install'),str(registry),str(PYTHON),str(ROOT/'tools/registry_validate.py'),str(ROOT/'target/debug/wb-registry-inspect'),str(ROOT/'target/vs2-fixtures/adversary-ui')],capture_output=True,check=True,timeout=90)
  c=Client(registry,self.workspace);self.clients.append(c);preview=c.op({'type':'prepare','utterance':'I need a mortgage calculator for a £5.00 loan over 25 years at 5%.'})['preview'];reply=c.op({'type':'approve','digest':preview['proposalDigest']});self.assertFalse(reply['ok']);self.assertTrue(reply['topologyCommitted'] and reply['topologyCheckpointPersisted']);self.assertFalse(reply['initializationSucceeded'] or reply['checkpointPersisted']);self.assertEqual(reply['nativeCode'],'COMPONENT_TRAP');state=c.desktop({'type':'select','roomId':reply['roomId']});self.assertTrue(state['accepted']);self.assertNotIn('payment',state['projection']['values']);saved=json.loads((self.workspace/'workspace.json').read_text());self.assertEqual(saved['rooms'][0]['room']['state'],{});self.assertEqual(saved['rooms'][0]['outputs'],{});c.close();c.record('worker-guest-trap')
 def test_op_015_handoff_lock_conflict_delivers_typed_outcome_before_fatal(self):
  c=self.client();preview=c.op({'type':'prepare','utterance':INTENT})['preview'];lock=self.workspace/'.workspace.lock';lock.rename(self.workspace/'.old-lock');fd=os.open(lock,os.O_CREAT|os.O_RDWR,0o600);fcntl.flock(fd,fcntl.LOCK_EX)
  try:reply=c.op({'type':'approve','digest':preview['proposalDigest']});self.assertFalse(reply['ok']);self.assertEqual(reply['code'],'STATE_CONFLICT');self.assertEqual(reply['nativeCode'],'WORKSPACE_BUSY');self.assertFalse(reply['topologyCommitted']);c.p.wait(timeout=10);self.assertEqual(c.p.returncode,1);self.assertFalse((self.workspace/'workspace.json').exists())
  finally:os.close(fd)
  c.record('worker-handoff-lock-conflict')
if __name__=='__main__':unittest.main(verbosity=2)
