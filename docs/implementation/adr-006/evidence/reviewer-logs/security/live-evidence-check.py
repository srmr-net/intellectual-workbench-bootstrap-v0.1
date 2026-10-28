"""Independent read-only review of sanitized evidence; never loads a credential."""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[6]
E=ROOT/'docs/implementation/adr-006/evidence'
P=E/'live-openrouter'
def load(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
f=load(P/'facts.json');consent=load(E/'openrouter-consent.json')
assert set(f['sha256'])=={p.name for p in P.iterdir() if p.is_file() and p.name!='facts.json'}
for name,digest in f['sha256'].items():assert sha(P/name)==digest,name
print('PASS: all',len(f['sha256']),'live evidence manifest entries and exact inventory verified')
r=load(P/'report.json');cold=load(P/'cold-report.json');publication=load(P/'report.json.preview.json');decision=load(P/'report.json.approval.json');preview=publication['preview']
assert consent['answer']=='Consent to this OpenRouter live smoke'
assert consent['endpoint']=='https://openrouter.ai/api/v1/chat/completions'
assert consent['model']=='nvidia/nemotron-3-ultra-550b-a55b:free'
assert consent['providerOnly']=='nvidia' and not consent['fallbacks'] and not consent['tools'] and not consent['automaticRetries']
assert consent['maxRequestsTotal']>=f['consentedRequestsUsed']==1
assert consent['maxOutputTokensPerRequest']==512 and consent['timeoutMsPerRequest']==20000
assert 'logs requests' in consent['disclosedDataUse'] and 'No provider-enforced JSON Schema' in consent['disclosedCapability']
assert decision=={'proposalDigest':preview['proposalDigest'],'decision':'approve'}
assert f['humanApproval']['proposalDigest']==preview['proposalDigest']
assert f['humanApproval']['answer']=='Approve this exact proposal'
actions=[s['action'] for s in r['samples']]
assert actions.index('remoteConsent')<actions.index('previewDraft')<actions.index('awaitHumanApproval')<actions.index('approve')
by=lambda name:next(s for s in r['samples'] if s['action']==name)
assert by('remoteConsent')['remoteConsent']
for name in ['previewDraft','awaitHumanApproval']:
 s=by(name);assert not s['remoteConsent'] and not s['consentChecked'] and not s['approveEnabled'] and s['state']['rooms']==[]
 assert s['operatorReply']['preview']['proposalDigest']==decision['proposalDigest']
assert by('approve')['operatorReply']['code']=='APPLIED' and by('approve')['operatorReply']['topologyCommitted']
assert preview['plan']['planVersion']=='0.1' and preview['plan']['expectedRoomRevision']==0
assert preview['plan']['requestedPermissions']==[] and len(preview['plan']['instances'])==1 and preview['requiresApproval']
print('PASS: scoped consent record, consumed consent, exact digest approval and preview-before-apply report ordering verified')
audit=by('previewDraft')['operatorReply']['providerAudit'];assert audit==f['providerAudit']
assert audit['attempts']==1 and audit['observation']['provider']=='openrouter-chat' and audit['observation']['model']==consent['model'] and audit['observation']['status']=='OK'
assert audit['observation']['inputTokens']==565 and audit['observation']['outputTokens']==159
assert audit['budget']['maxCalls']==1 and audit['budget']['maxOutputTokens']==512 and audit['budget']['timeoutMs']==20000
assert not publication['providerState']['providerEnforcedSchema'] and publication['providerState']['transport']=='https-production'
assert by('approve')['state']['projection']['values']['payment']=='1461.48'
assert r['finalState']['projection']['values']['payment']=='1753.77'
assert by('operatorStatus')['providerState']['remoteAttempts']=='1' and by('operatorStatus')['operatorReply']['providerCalls']=='1'
assert cold['finalState']['rooms']==r['finalState']['rooms']
assert cold['finalState']['projection']['values']['payment']=='1753.77' and cold['finalState']['invocations']=='0'
cs=next(s for s in cold['samples'] if s['action']=='operatorStatus');assert cs['providerState']['remoteAttempts']=='0' and cs['operatorReply']['providerCalls']=='0'
assert 'OFFLINE_IPV4_IPV6_DENIED' in (P/'cold-stdout.txt').read_text()
assert len({r['pid'],r['workerProcessPid'],cold['pid'],cold['workerProcessPid']})==4
for report in [r,cold]:assert report['failure']=='' and report['exitCode']==0 and report['initialGuestInvocations']=='0' and report['initialProviderCalls']=='0' and report['guiThread']
print('PASS: actual HTTPS profile evidence, bounded usage, accepted execution, reactive call count and isolated offline cold-process records verified')
manifest=load(ROOT/'target/ui-fixtures/mortgage-ui/component.json');room=r['finalState']['rooms'][0]
assert sha(ROOT/'target/ui-fixtures/mortgage-ui/logic.wasm')==manifest['runtime']['sha256']==f['installedPin']['sha256']
assert f['installedPin']==preview['plan']['instances'][0]['component']==room['component']
assert room['ui']['sha256']==sha(ROOT/'target/ui-fixtures/mortgage-ui/ui.json')
assert room['ui']['schemaSha256']==sha(ROOT/'schemas/ui.schema.json')
assert sha(ROOT/'target/llm-live/debug/wb-live-worker')==f['workerSha256']
assert sha(ROOT/'target/llm-qt-build/workbench-live-provider')==f['appSha256']==r['binarySha256']==cold['binarySha256']
assert not f['rawHttpBodiesArchived'] and 'No exact-spend claim' in f['cost']
print('PASS: current production binaries, manifest/WASM exact pin, UI/schema hashes and no-exact-cost claim verified')
print('LIMITATION: consent/human decision are recorded session attestations plus driver ordering, not independently witnessed chat or an authenticated timestamp service.')
print('LIMITATION: no actual key was read; exact-key scan result is implementer-attested. Structural/privacy review and screenshot inspection found only scoped synthetic UI data, not raw HTTP envelopes.')
print('LIMITATION: provider error negatives remain mocked; remote cancellation/billing cessation and provider-retention guarantees are not claimed.')
