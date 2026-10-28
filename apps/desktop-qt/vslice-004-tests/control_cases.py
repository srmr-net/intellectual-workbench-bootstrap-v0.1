"""Additional native ingress adversaries for the real product worker."""
import importlib.util,json,os,subprocess,unittest.mock
from pathlib import Path
from contract import ROOT
spec=importlib.util.spec_from_file_location('accepted_control',ROOT/'apps/desktop-qt/operator-tests/worker_cases.py');base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
base.WORKER=ROOT/'target/debug/wb-vslice004-worker'
def exercise(test,case):
 workspace=test.path/('native-'+case);original=subprocess.Popen
 def popen(*args,**kw):
  env=kw['env']
  for name in list(env):
   if name.startswith(('WB_LLM_','WB_VS4_','OPENROUTER_','OPENAI_')):env.pop(name)
  env.update(WB_VS4_TEST_MODE='success',WB_LLM_SECRET_FILE=str(test.key))
  return original(*args,**kw)
 with unittest.mock.patch.object(subprocess,'Popen',popen):c=base.Client(test.registry,workspace)
 def call(protocol,command):
  c.counter+=1;token='native-'+str(c.counter);c.send({'protocol':protocol,'token':token,'command':command});r=c.read(True);test.assertEqual(r['token'],token);return r
 configure=lambda:call('provider.1',{'type':'configure','provider':'openrouter-chat','consent':False})
 review=lambda:call('ux.1',{'type':'review','utterance':base.INTENT})['disclosure']
 send=lambda d:call('ux.1',{'type':'send','reviewId':d['reviewId'],'reviewDigest':d['reviewDigest']})
 try:
  test.assertTrue(configure()['ok'])
  if case=='bypass':
   test.assertFalse(call('provider.1',{'type':'configure','provider':'openrouter-chat','consent':True})['ok']);test.assertEqual(c.op({'type':'prepare','utterance':base.INTENT})['code'],'DISCLOSURE_REQUIRED');test.assertEqual(call('provider.1',{'type':'status'})['remoteAttempts'],'0')
  elif case=='source':
   d=review();test.assertTrue(call('ux.1',{'type':'credential','path':str(test.key)})['ok']);test.assertEqual(send(d)['code'],'DISCLOSURE_EXPIRED');test.assertEqual(call('provider.1',{'type':'status'})['remoteAttempts'],'0')
   d=review();test.key.chmod(0o644)
   try:r=send(d);test.assertFalse(r['ok']);test.assertEqual(call('provider.1',{'type':'status'})['remoteAttempts'],'0')
   finally:test.key.chmod(0o600)
  elif case=='replay':
   d=review();test.assertTrue(call('ux.1',{'type':'status'})['ok']);r=send(d);test.assertTrue(r['ok'],r);test.assertEqual(send(d)['code'],'DISCLOSURE_EXPIRED');test.assertEqual(call('provider.1',{'type':'status'})['remoteAttempts'],'1')
  elif case=='durable':
   d=review();r=send(d);test.assertTrue(r['ok'],r);a=c.op({'type':'approve','digest':r['preview']['proposalDigest']});test.assertTrue(a['ok']);c.desktop({'type':'select','roomId':a['roomId']});d=review();path=workspace/'workspace.json';saved=json.loads(path.read_text());saved['revision']+=1;saved['rooms'][0]['pinned']=True;saved.pop('contentSha256');import hashlib;saved['contentSha256']=hashlib.sha256(json.dumps(saved,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest();path.write_text(json.dumps(saved));r=send(d);test.assertFalse(r['ok']);test.assertEqual(call('provider.1',{'type':'status'})['remoteAttempts'],'1')
  test.assertEqual(c.op({'type':'status'})['providerCalls'], '0' if case=='bypass' else '1' if case in ['source','replay'] else '2')
  # Quit must not write over externally replaced durable state.
  if case=='durable':c.kill()
  else:c.close()
  directory=Path(os.environ.get('WB_VS4_EVIDENCE_DIR',ROOT/'target/abi-evidence/vs4'))/'control';directory.mkdir(parents=True,exist_ok=True);(directory/(case+'.json')).write_text(json.dumps({'pid':c.p.pid,'records':c.records},indent=2)+'\n')
 finally:c.kill()
