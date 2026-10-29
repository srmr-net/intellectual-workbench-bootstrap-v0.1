"""Exercise actual framed production/fixture workers without network or real keys."""
import json, os, struct, subprocess, unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
FIXTURE=ROOT/'target/operator-chat-fixture/debug/wb-operator-chat'
PRODUCTION=ROOT/'target/operator-chat-native/debug/wb-operator-chat'
class Worker:
    def __init__(self, fixture=True):
        self.p=subprocess.Popen([str(FIXTURE if fixture else PRODUCTION)]+(['--fixture'] if fixture else []),stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env={'PATH':'/usr/bin:/bin'})
        self.sequence=0
        self.initial=self.read()
    def read(self):
        prefix=self.p.stdout.read(4)
        if len(prefix)!=4:raise AssertionError('missing frame')
        n=struct.unpack('>I',prefix)[0]
        assert 0<n<=4*1024*1024
        return json.loads(self.p.stdout.read(n))
    def send(self, command):
        self.sequence+=1
        raw=json.dumps(dict(protocol='chat.1',token=f't-{self.sequence}',command=command)).encode()
        self.raw(raw)
        return self.read()
    def raw(self, raw):
        self.p.stdin.write(struct.pack('>I',len(raw))+raw);self.p.stdin.flush()
    def close(self):
        if self.p.poll() is None:
            self.send(dict(type='quit'))
            self.p.wait(timeout=5)
        assert self.p.returncode==0
        assert self.p.stderr.read()==b''
        self.p.stdin.close();self.p.stdout.close();self.p.stderr.close()
class ProcessTests(unittest.TestCase):
    def test_production_starts_off_and_no_mock_flag(self):
        w=Worker(False)
        try:
            self.assertFalse(w.initial['state']['enabled']);self.assertEqual(w.initial['state']['calls'],0)
            if os.environ.get('WB_CHAT_EVIDENCE_DIR'):
                target=Path(os.environ['WB_CHAT_EVIDENCE_DIR']);target.mkdir(parents=True,exist_ok=True)
                (target/'production-profile.json').write_text(json.dumps({k:w.initial['state'][k] for k in ['enabled','available','calls','model','endpoint','contextTurnLimit']},indent=2)+'\n')
            w.send(dict(type='new'))
            r=w.send(dict(type='send',sessionId='chat-1',text='Hello'))
            self.assertEqual(r['code'],'CONSENT_REQUIRED');self.assertEqual(r['state']['calls'],0)
        finally:w.close()
        p=subprocess.run([str(PRODUCTION),'--fixture'],env={'PATH':'/usr/bin:/bin'},capture_output=True,timeout=5)
        self.assertNotEqual(p.returncode,0);self.assertEqual(p.stdout,b'');self.assertEqual(p.stderr,b'')
    def test_real_process_multiturn_and_cancel(self):
        w=Worker()
        try:
            w.send(dict(type='new'));w.send(dict(type='configure',enabled=True,credentialPath=None))
            self.assertEqual(w.send(dict(type='send',sessionId='chat-1',text='Remember blue'))['kind'],'started')
            self.assertEqual(w.read()['kind'],'completed')
            w.send(dict(type='send',sessionId='chat-1',text='Which colour?'))
            r=w.read();self.assertIn('Remember blue',r['state']['messages'][-1]['text']);self.assertEqual(r['state']['calls'],2)
            w.send(dict(type='send',sessionId='chat-1',text='Cancel now'))
            r=w.send(dict(type='cancel'));self.assertFalse(r['state']['busy']);self.assertEqual(r['state']['messages'][-1]['status'],'cancelled')
        finally:w.close()
    def test_duplicate_unknown_and_oversized_frames_are_closed(self):
        for raw in [b'{"protocol":"chat.1","protocol":"chat.1","token":"t","command":{"type":"new"}}',b'{"protocol":"chat.1","token":"t","command":{"type":"new","execute":"anything"}}',b'x'*32769]:
            with self.subTest(size=len(raw)):
                w=Worker()
                if len(raw)>32768:
                    w.p.stdin.write(struct.pack('>I',len(raw)));w.p.stdin.flush()
                else:w.raw(raw)
                r=w.read();self.assertEqual(r['kind'],'fatal');self.assertEqual(r['code'],'INVALID_FRAME');w.p.wait(timeout=5);w.close()
if __name__=='__main__':unittest.main()
