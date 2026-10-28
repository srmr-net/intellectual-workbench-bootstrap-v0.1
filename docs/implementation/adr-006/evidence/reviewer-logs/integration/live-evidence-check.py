"""Read-only independent verification of archived consented live evidence; no credentials/network."""
import hashlib,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[6]
P=ROOT/'docs/implementation/adr-006/evidence/live-openrouter'
load=lambda n:json.loads((P/n).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda v:json.dumps(v,ensure_ascii=False,separators=(',',':')).encode()
f=load('facts.json');r=load('report.json');cold=load('cold-report.json');pv=load('report.json.preview.json');v=pv['preview'];approval=load('report.json.approval.json')
for name,h in f['sha256'].items():assert sha((P/name).read_bytes())==h,name
assert all(k in f['sha256'] for k in ['report.json','cold-report.json','report.json.preview.json','report.json.approval.json','production-build.log','versions.json'])
assert f['source']=='cd7b29b31cc5f2ec5f9b4fb2ba9699125dc864d0'
subprocess.run(['git','merge-base','--is-ancestor',f['source'],'HEAD'],cwd=ROOT,check=True)
assert approval=={'proposalDigest':v['proposalDigest'],'decision':'approve'}
assert f['humanApproval']['proposalDigest']==v['proposalDigest'] and f['humanApproval']['answer']=='Approve this exact proposal'
assert v['initialInputs']==f['initialInputs']=={'annual_rate_percent':'5','principal':'250000','term_months':300}
assert sha(canonical(v['initialInputs']))==v['initialInputDigest']
p=v['plan'];assert p['planVersion']=='0.1' and p['expectedRoomRevision']==0 and p['requestedPermissions']==[] and p['connections']==[] and len(p['instances'])==1
pin=p['instances'][0]['component'];assert pin==f['installedPin'];assert all(v['validation'][k] for k in ['canonicalPlan','exactPin','initialInputSchema','admittedUiBindings','purePermissions'])
# Reproduce accepted Rust Serialize field ordering, then policy-revision binding.
plan={k:p[k] for k in ['planVersion','roomId','expectedRoomRevision','instances','connections','ui','requestedPermissions']}
plan['instances']=[{'instanceId':i['instanceId'],'component':{k:i['component'][k] for k in ['id','version','sha256']}} for i in p['instances']]
ui=r['finalState']['rooms'][0]['ui']
base=sha(canonical(['apply-and-initialize',v['proposalId'],v['expiresInSeconds']*1_000_000_000,plan,v['initialInputs'],ui['sha256'],ui['schemaSha256']]))
assert sha(canonical([base,v['policyRevision']]))==v['proposalDigest']
for doc in [r,cold]:
 assert doc['exitCode']==0 and not doc['failure'] and doc['guiThread'] and doc['pid']!=doc['workerProcessPid']==doc['finalState']['pid']
 assert doc['binarySha256']==f['appSha256'] and doc['initialProviderCalls']=='0' and doc['initialGuestInvocations']=='0'
assert len({r['pid'],r['workerProcessPid'],cold['pid'],cold['workerProcessPid']})==4
assert [r['pid'],r['workerProcessPid'],cold['pid'],cold['workerProcessPid']]==[f[k] for k in ['desktopPid','workerPid','coldDesktopPid','coldWorkerPid']]
preview=next(s for s in r['samples'] if s['action']=='previewDraft');applied=next(s for s in r['samples'] if s['action']=='approve');edited=next(s for s in r['samples'] if s['action']=='input')
assert preview['operatorReply']['preview']==v
assert applied['operatorReply']['code']=='APPLIED' and applied['operatorReply']['initialInvocations']=='1'
assert applied['operatorReply']['topologyCommitted'] and applied['operatorReply']['initializationSucceeded'] and applied['operatorReply']['checkpointPersisted']
assert applied['state']['projection']['values']['payment']==f['initialPayment']=='1461.48'
assert edited['state']['projection']['values']['payment']==f['reactivePayment']=='1753.77'
for s in r['samples'][3:]:assert s['providerState']['remoteAttempts']=='1'
status=next(s for s in r['samples'] if s['action']=='operatorStatus');assert status['operatorReply']['providerCalls']=='1'
assert preview['operatorReply']['providerAudit']==f['providerAudit']
assert f['providerAudit']['observation']['provider']=='openrouter-chat' and f['providerAudit']['observation']['model']=='nvidia/nemotron-3-ultra-550b-a55b:free' and f['providerAudit']['observation']['status']=='OK'
assert f['providerAudit']['attempts']==1 and f['providerAudit']['observation']['outputTokens']<=512
assert pv['providerState']['transport']=='https-production' and pv['providerState']['providerEnforcedSchema'] is False
assert cold['finalState']['rooms']==r['finalState']['rooms'] and cold['finalState']['rooms'][0]['pinned']
assert cold['finalState']['projection']['values']==r['finalState']['projection']['values'] and cold['finalState']['invocations']=='0'
for s in cold['samples']:assert s['providerState']['remoteAttempts']=='0' and s['operatorReply']['providerCalls']=='0'
assert 'OFFLINE_IPV4_IPV6_DENIED' in (P/'cold-stdout.txt').read_text()
assert load('launcher.json')['returncode']==0 and load('launcher.json')['maxCalls']==1
assert sha((ROOT/'target/llm-qt-build/workbench-live-provider').read_bytes())==f['appSha256']
assert sha((ROOT/'target/llm-live/debug/wb-live-worker').read_bytes())==f['workerSha256']
assert 'Finished `dev` profile' in (P/'production-build.log').read_text()
assert set(load('versions.json'))=={'rustc','cargo','qt','python'}
print(json.dumps({'verified':True,'source':f['source'],'hashedFiles':len(f['sha256']),'proposalDigest':v['proposalDigest'],'recomputedProposalDigest':True,'exactPin':pin,'initialPayment':'1461.48','reactivePayment':'1753.77','remoteAttempts':1,'providerCalls':1,'distinctProcesses':4,'coldProviderCalls':0,'coldGuestCalls':0,'coldNetworkDenied':True,'providerEnforcedSchema':False,'rawProviderBodiesArchived':False,'limitations':['Artifact consistency verified; user chat consent/approval assertion attributed to archived host record, not independently queried account.','No exact billing or provider-retention guarantee; no real refusal/timeout observed; mocked negatives remain separate.','Accepted topology-then-ingress boundary remains non-atomic.']},indent=2))
