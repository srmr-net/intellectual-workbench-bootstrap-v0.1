"""Real app executable/process conformance. No mocked Qt tree or local math."""
from pathlib import Path
import hashlib,json,os,shutil,subprocess,sys,tempfile,unittest
ROOT=Path(__file__).resolve().parents[3]
APP=ROOT/'target/desktop-build/workbench-desktop'
NATIVE=ROOT/'target/debug'
PYTHON=os.environ.get('REGISTRY_PYTHON',sys.executable)
EVIDENCE=Path(os.environ.get('WB_VS2_EVIDENCE_DIR',ROOT/'target/abi-evidence/vs2'))
MORTGAGE='org.workbench.example.mortgage-ui'
ALTERNATE='org.workbench.example.alternate-ui'
ADVERSARY='org.workbench.example.vs2-adversary-ui'
def open_(id=MORTGAGE):return {'type':'open','id':id}
def input_(id,text,**kw):return {'type':'input','id':id,'text':text,**kw}
def expect(**kw):return {'type':'expect',**kw}
def calculate(p='250000.00',r='5.00',t='300',ids=('principal','rate','term')):
 return [input_(ids[0],p),input_(ids[1],r),input_(ids[2],t)]
QUIT={'type':'quit'}

class DesktopProcesses(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.base=tempfile.TemporaryDirectory(prefix='wb-vs2-installed-');cls.base_path=Path(cls.base.name)
  packages=[ROOT/'target/ui-fixtures/mortgage-ui',ROOT/'target/ui-fixtures/alternate-ui',ROOT/'target/vs2-fixtures/adversary-ui']
  command=[str(NATIVE/'wb-desktop-install'),str(cls.base_path/'registry'),str(PYTHON),str(ROOT/'tools/registry_validate.py'),str(NATIVE/'wb-registry-inspect'),*map(str,packages)]
  result=subprocess.run(command,capture_output=True,text=True,timeout=120,check=True)
  EVIDENCE.mkdir(parents=True,exist_ok=True);(EVIDENCE/'admission.jsonl').write_text(result.stdout)
 @classmethod
 def tearDownClass(cls):cls.base.cleanup()
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory(prefix='wb-vs2-process-');self.path=Path(self.temp.name)
  self.registry=self.path/'registry';shutil.copytree(self.base_path/'registry',self.registry)
  self.workspace=self.path/'workspace';self.runs=0
 def tearDown(self):self.temp.cleanup()
 def launch(self,actions,offline=False):
  self.runs+=1;directory=self.path/f'run-{self.runs}';directory.mkdir();script=directory/'actions.json';script.write_text(json.dumps(actions))
  report=directory/'report.json'
  command=[str(APP),str(NATIVE/'wb-desktop-worker'),str(self.registry),str(self.workspace),str(PYTHON),str(ROOT/'tools/registry_validate.py'),str(NATIVE/'wb-registry-inspect'),'--validation-actions',str(script),'--validation-report',str(report)]
  if offline:command.insert(0,str(NATIVE/'wb-desktop-offline'))
  env=os.environ.copy()
  for key in list(env):
   if key in ('QT_QPA_PLATFORMTHEME','DISPLAY','WAYLAND_DISPLAY') or key.startswith(('OPENAI_','ANTHROPIC_','AZURE_OPENAI_','OLLAMA_')):env.pop(key,None)
  env.update(QT_QPA_PLATFORM='offscreen',QT_QUICK_BACKEND='software',QT_LINUX_ACCESSIBILITY_ALWAYS_ON='1',HTTP_PROXY='http://127.0.0.1:9',HTTPS_PROXY='http://127.0.0.1:9',NO_PROXY='')
  process=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,env=env)
  try:stdout,stderr=process.communicate(timeout=150)
  except subprocess.TimeoutExpired:process.kill();stdout,stderr=process.communicate();self.fail('real desktop process timed out: '+stderr)
  (directory/'stdout.txt').write_text(stdout);(directory/'stderr.txt').write_text(stderr);(directory/'command.json').write_text(json.dumps(command,indent=2)+'\n')
  self.assertTrue(report.exists(),stderr)
  result=json.loads(report.read_text());(directory/'launcher.json').write_text(json.dumps({'pid':process.pid,'returncode':process.returncode,'offline':offline},indent=2)+'\n')
  if (self.workspace/'workspace.json').is_file() and not (self.workspace/'workspace.json').is_symlink():shutil.copyfile(self.workspace/'workspace.json',directory/'workspace.json')
  destination=EVIDENCE/'process'/self._testMethodName/f'run-{self.runs}'
  if destination.exists():shutil.rmtree(destination)
  shutil.copytree(directory,destination)
  self.assertEqual(process.returncode,0,(result.get('failure'),stderr));self.assertEqual(result['failure'],'');self.assertEqual(result['pid'],process.pid)
  self.assertEqual(result['exitCode'],0);self.assertEqual(result['platform'],'offscreen');self.assertTrue(result['guiThread'])
  self.assertEqual(result['binarySha256'],hashlib.sha256(APP.read_bytes()).hexdigest())
  self.assertEqual(result['workerProcessPid'],result['finalState']['pid'])
  if offline:self.assertIn('OFFLINE_IPV4_IPV6_DENIED',stdout)
  return result
 def snapshot(self):return (self.workspace/'workspace.json').read_bytes()
 def test_vs2_001_documented_real_desktop_launch_and_clean_exit(self):
  r=self.launch([expect(rooms=0,ready=False),QUIT]);self.assertGreater(r['pid'],0);self.assertTrue(r['qtVersion'].startswith('6.'));self.assertEqual(r['finalState']['invocations'],'0')
 def test_vs2_002_real_three_column_shell(self):
  r=self.launch([QUIT]);columns=r['samples'][0]['columns'];self.assertEqual([c['name'] for c in columns],['leftPinnedTools','activityContext','semanticCanvas']);self.assertTrue(all(c['visible'] and c['width']>0 for c in columns));self.assertTrue(columns[0]['x']<columns[1]['x']<columns[2]['x'])
 def test_vs2_003_installed_exact_pin_picker(self):
  r=self.launch([open_(),expect(ready=True,rooms=1),QUIT]);s=r['finalState'];self.assertEqual(s['rooms'][0]['component']['id'],MORTGAGE);self.assertIn(s['rooms'][0]['component'],[v['pin'] for v in s['installed']]);self.assertEqual(s['projection']['key']['uiId'],'ui.json#'+s['rooms'][0]['ui']['sha256'])
 def test_vs2_005_unchanged_generic_renderer_alternate_metadata(self):
  r=self.launch([open_(ALTERNATE),*calculate('120000.00','0','240',('amount','interest','duration')),expect(values={'result':'500.00'}),QUIT]);controls=r['samples'][-2]['controls'];self.assertEqual([c['id'] for c in controls],['amount','interest','duration','result']);self.assertTrue(all(c['name'] for c in controls));self.assertEqual([c['role'] for c in controls],[42,42,42,41])
 def test_vs2_009_field_diagnostics_without_compute_or_revision(self):
  r=self.launch([open_(),input_('principal','-1.00'),expect(invocations='0',accessibleError='principal',error='SCHEMA VIOLATION'),input_('principal','250000.00'),input_('rate','5.00'),input_('term','0'),expect(invocations='0',accessibleError='term'),input_('term','300'),expect(values={'payment':'1461.48'},invocations='1'),input_('principal','-2.00'),expect(values={'payment':'1461.48'},invocations='1',accessibleError='principal'),QUIT]);invalid=[s for s in r['samples'] if s['action']=='expect'];self.assertEqual(invalid[0]['state']['projection']['roomRevision'],'1');self.assertEqual(invalid[-1]['state']['projection']['roomRevision'],'2');saved=json.loads(self.snapshot());self.assertEqual(saved['rooms'][0]['room']['state']['component.calculate']['principal'],'250000.00')
 def test_vs2_013_real_guest_failure_retains_committed_readout(self):
  r=self.launch([open_(ADVERSARY),*calculate('1.00','0','1'),expect(values={'payment':'123.45'}),input_('principal','5.00'),expect(values={'payment':'123.45'},error='COMPONENT TRAP'),input_('principal','4.00'),expect(values={'payment':'123.45'},error='SCHEMA VIOLATION'),QUIT]);self.assertTrue(r['finalState']['lastTrace']['checkpointPreserved']);self.assertEqual(json.loads(self.snapshot())['rooms'][0]['room']['state']['component.calculate']['principal'],'1.00')
 def test_vs2_014_rapid_room_switch_and_teardown(self):
  actions=[open_(),*calculate(),{'type':'pin'},open_(ALTERNATE),*calculate('120000.00','0','240',('amount','interest','duration')),{'type':'pin'}]
  for _ in range(3):actions += [{'type':'select','index':0},expect(values={'payment':'1461.48'},invocations='2'),{'type':'select','index':1},expect(values={'result':'500.00'},invocations='2')]
  r=self.launch([*actions,QUIT]);self.assertTrue(all(s['guiThread'] for s in r['samples']));self.assertEqual(len({v['component']['id'] for v in r['finalState']['rooms']}),2)
 def test_vs2_015_keyboard_tab_accessibility_and_focus_retention(self):
  r=self.launch([open_(),input_('principal','250000.00',tabTo='rate'),input_('rate','5.00',tabTo='term'),input_('term','300'),expect(values={'payment':'1461.48'}),{'type':'focus','id':'principal','cursor':3},{'type':'backgroundEdit','id':'rate','text':'0'},expect(values={'payment':'833.33'},focus='principal',cursor=3),QUIT]);controls=r['samples'][0]['controls'];self.assertEqual([c['name'] for c in controls],['Principal','Annual interest rate','Term (months)','Monthly repayment']);self.assertTrue(all(c['focusable'] for c in controls[:3]))
 def test_vs2_017_pin_unpin_and_canvas_selection_persist(self):
  a=self.launch([open_(),{'type':'pin'},open_(ALTERNATE),{'type':'pin'},{'type':'select','index':0},{'type':'pin'},expect(pinned=False,rooms=2),QUIT]);b=self.launch([expect(pinned=False,rooms=2,invocations='0'),QUIT]);self.assertEqual(a['finalState']['selectedRoom'],b['finalState']['selectedRoom']);self.assertEqual([r['pinned'] for r in b['finalState']['rooms']],[False,True])
 def test_vs2_018_cold_process_restart_exact_pins_no_compute(self):
  a=self.launch([open_(),*calculate(),{'type':'pin'},QUIT]);original=self.snapshot();b=self.launch([expect(values={'principal':'250000.00','rate':'5.00','term':'300','payment':'1461.48'},invocations='0',pinned=True),QUIT]);self.assertNotEqual(a['pid'],b['pid']);self.assertNotEqual(a['finalState']['pid'],b['finalState']['pid']);self.assertEqual(a['finalState']['rooms'],b['finalState']['rooms']);self.assertEqual(self.snapshot(),original)
 def test_vs2_019_repeated_clean_restart_snapshot_bytes_stable(self):
  results=[self.launch([open_(),*calculate('120000.00','0','240'),{'type':'pin'},QUIT])];original=self.snapshot()
  for _ in range(2):results.append(self.launch([expect(values={'payment':'500.00'},invocations='0'),QUIT]));self.assertEqual(self.snapshot(),original)
  self.assertEqual(len({r['pid'] for r in results}),3);self.assertEqual(json.loads(original)['formatVersion'],1)
 def test_vs2_020_recovery_is_visible_and_non_destructive(self):
  self.launch([open_(),*calculate(),{'type':'pin'},QUIT]);original=self.snapshot()
  for corrupt in [b'{',b'{"formatVersion":1,"formatVersion":1}',original.replace(b'"formatVersion":1',b'"formatVersion":9')]:
   (self.workspace/'workspace.json').write_bytes(corrupt);self.launch([expect(ready=False,error='Recovery required'),QUIT]);self.assertEqual(self.snapshot(),corrupt)
  (self.workspace/'workspace.json').write_bytes(original)
  for logic in (self.registry/'objects').glob('*/payload/logic.wasm'):logic.write_bytes(b'tampered')
  self.launch([expect(ready=False,error='Recovery required'),QUIT]);self.assertEqual(self.snapshot(),original)
 def test_vs2_021_snapshot_symlink_and_oversize_recovery(self):
  self.launch([open_(),QUIT]);path=self.workspace/'workspace.json';original=path.read_bytes();outside=self.path/'outside.json';outside.write_bytes(original);path.unlink();path.symlink_to(outside)
  self.launch([expect(ready=False,error='Recovery required'),QUIT]);self.assertTrue(path.is_symlink());self.assertEqual(outside.read_bytes(),original);path.unlink();path.write_bytes(b' '*(1024*1024+1));path.chmod(0o600)
  self.launch([expect(ready=False,error='Recovery required'),QUIT]);self.assertEqual(path.stat().st_size,1024*1024+1)
 def test_vs2_022_actual_offline_process_restore(self):
  self.launch([open_(),*calculate(),{'type':'pin'},QUIT]);original=self.snapshot();r=self.launch([expect(values={'payment':'1461.48'},invocations='0',pinned=True),QUIT],offline=True);self.assertEqual(r['finalState']['invocations'],'0');self.assertEqual(self.snapshot(),original)

if __name__=='__main__':unittest.main(verbosity=2)
