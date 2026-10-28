"""Explicit consented live Qt smoke. NEVER run in CI. No key contents are inspected here."""
import argparse,json,os,shutil,subprocess,sys,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
INTENT='I need a mortgage calculator for a £250,000 loan over 25 years at 5%.'
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--credential-file',type=Path,required=True);p.add_argument('--consented',action='store_true');p.add_argument('--output',type=Path,required=True);a=p.parse_args()
 if not a.consented or (os.environ.get('GITHUB_ACTIONS')=='true' or os.environ.get('CI','').lower() in ('1','true','yes')):p.error('explicit scoped remote consent required; forbidden in hosted CI')
 out=a.output.resolve();out.mkdir(parents=True,exist_ok=False);registry=out/'private-registry';workspace=out/'private-workspace'
 python=ROOT/'target/vs2-venv/bin/python';native=ROOT/'target/debug';worker=ROOT/'target/llm-live/debug/wb-live-worker';app=ROOT/'target/llm-qt-build/workbench-live-provider'
 subprocess.run([str(native/'wb-desktop-install'),str(registry),str(python),str(ROOT/'tools/registry_validate.py'),str(native/'wb-registry-inspect'),str(ROOT/'target/ui-fixtures/mortgage-ui')],capture_output=True,check=True,timeout=120)
 actions=[{'type':'selectProvider','remote':True},{'type':'draftIntent','text':INTENT},{'type':'remoteConsent','consent':True},{'type':'previewDraft'},{'type':'awaitHumanApproval'},{'type':'approve'},{'type':'expect','values':{'payment':'1461.48'}},{'type':'input','id':'principal','text':'300000'},{'type':'expect','values':{'payment':'1753.77'}},{'type':'pin','pinned':True},{'type':'operatorStatus'},{'type':'quit'}]
 script=out/'actions.json';script.write_text(json.dumps(actions));report=out/'report.json'
 env=os.environ.copy()
 for key in list(env):
  if key.startswith(('OPENAI_','OPENROUTER_','WB_LLM_','ANTHROPIC_','AZURE_OPENAI_')) or key in ['DISPLAY','WAYLAND_DISPLAY','QT_QPA_PLATFORMTHEME']:env.pop(key,None)
 env.update(QT_QPA_PLATFORM='offscreen',QT_QUICK_BACKEND='software',WB_LLM_SECRET_FILE=str(a.credential_file.expanduser().resolve()),WB_LLM_MODEL='nvidia/nemotron-3-ultra-550b-a55b:free',WB_LLM_PROVIDER='openrouter-chat')
 command=[str(app),str(worker),str(registry),str(workspace),str(python),str(ROOT/'tools/registry_validate.py'),str(native/'wb-registry-inspect'),'--validation-actions',str(script),'--validation-report',str(report)]
 with (out/'stdout.txt').open('w') as stdout,(out/'stderr.txt').open('w') as stderr:
  process=subprocess.Popen(command,env=env,stdout=stdout,stderr=stderr)
  print(json.dumps({'desktopPid':process.pid,'preview':str(report)+'.preview.json','decision':str(report)+'.approval.json'}),flush=True)
  try:code=process.wait(timeout=150)
  except subprocess.TimeoutExpired:process.kill();process.wait();raise RuntimeError('live desktop deadline expired') from None
 (out/'launcher.json').write_text(json.dumps({'pid':process.pid,'returncode':code,'endpoint':'https://openrouter.ai/api/v1/chat/completions','model':'nvidia/nemotron-3-ultra-550b-a55b:free','maxCalls':1,'maxOutputTokens':512,'timeoutMs':20000},indent=2)+'\n')
 if code:raise SystemExit('Live smoke incomplete; inspect redacted status/report, no automatic retry')
 r=json.loads(report.read_text());assert r['finalState']['projection']['values']['payment']=='1753.77';assert next(s for s in r['samples'] if s['action']=='operatorStatus')['providerState']['remoteAttempts']=='1'
 cold_actions=out/'cold-actions.json';cold_actions.write_text(json.dumps([{'type':'expect','ready':True,'values':{'payment':'1753.77'},'invocations':'0'},{'type':'operatorStatus'},{'type':'quit'}]));cold_report=out/'cold-report.json'
 cold_env=env.copy();cold_env.pop('WB_LLM_SECRET_FILE',None);cold_env['WB_OPERATOR_PROVIDER']='disabled'
 cold_command=[str(native/'wb-desktop-offline'),*command];cold_command[cold_command.index(str(script))]=str(cold_actions);cold_command[cold_command.index(str(report))]=str(cold_report)
 with (out/'cold-stdout.txt').open('w') as stdout,(out/'cold-stderr.txt').open('w') as stderr: cold=subprocess.run(cold_command,env=cold_env,stdout=stdout,stderr=stderr,timeout=60,check=True)
 restored=json.loads(cold_report.read_text());assert restored['pid']!=r['pid'] and restored['workerProcessPid']!=r['workerProcessPid'];assert restored['finalState']['rooms']==r['finalState']['rooms'] and restored['finalState']['invocations']=='0';assert restored['samples'][1]['providerState']['remoteAttempts']=='0';assert restored['samples'][1]['operatorReply']['providerCalls']=='0';assert 'OFFLINE_IPV4_IPV6_DENIED' in (out/'cold-stdout.txt').read_text()
 print('LIVE execution, reactive edit and offline cold-process display passed',flush=True)
if __name__=='__main__':main()
