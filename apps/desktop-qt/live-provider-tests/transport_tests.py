"""No sockets: exercise the actual trusted HTTPS helper with deterministic I/O."""
import importlib.util,socket,ssl,unittest
from pathlib import Path
from unittest.mock import patch,MagicMock
SOURCE=Path(__file__).resolve().parents[1]/'live-provider/host/transport.py'
spec=importlib.util.spec_from_file_location('trusted_transport',SOURCE);transport=importlib.util.module_from_spec(spec);spec.loader.exec_module(transport)
class Transport(unittest.TestCase):
 def record(self,address):return [(socket.AF_INET6 if ':' in address else socket.AF_INET,socket.SOCK_STREAM,6,'',(address,443))]
 def test_llm_004_private_and_mixed_dns_fail_closed(self):
  for ip in ['127.0.0.1','10.0.0.1','169.254.169.254','192.168.1.1','0.0.0.0','224.0.0.1','::1','fc00::1','fe80::1','::ffff:127.0.0.1']:
   with self.subTest(ip=ip),self.assertRaises(ValueError):transport.public_addresses(self.record(ip))
  with self.assertRaises(ValueError):transport.public_addresses(self.record('8.8.8.8')+self.record('127.0.0.1'))
  self.assertEqual(transport.public_addresses(self.record('8.8.8.8')),[(socket.AF_INET,('8.8.8.8',443))])
 def send(self,status=200,headers=None,data=b'{}',error=None,provider="openai-responses"):
  raw=MagicMock();context=MagicMock();connection=MagicMock();reply=MagicMock();reply.status=status
  headers={'Content-Type':'application/json','Content-Length':str(len(data)),**(headers or {})};reply.getheader.side_effect=lambda key,default=None:headers.get(key,default);reply.read.return_value=data
  connection.getresponse.return_value=reply
  with patch.object(socket,'getaddrinfo',return_value=self.record('8.8.8.8')) as dns,patch.object(socket,'socket',return_value=raw),patch.object(ssl,'create_default_context',return_value=context),patch.object(transport.http.client,'HTTPSConnection',return_value=connection):
   if error:context.wrap_socket.side_effect=error
   result=transport.send(b'{"synthetic":true}',b'fixture-key',100,1000,provider)
  return result,(dns,raw,context,connection,reply)
 def test_llm_004_verified_tls_fixed_origin_no_reresolve(self):
  (metadata,body),(dns,raw,context,connection,reply)=self.send()
  dns.assert_called_once_with('api.openai.com',443,type=socket.SOCK_STREAM);raw.connect.assert_called_once_with(('8.8.8.8',443));context.wrap_socket.assert_called_once_with(raw,server_hostname='api.openai.com');self.assertEqual(context.minimum_version,ssl.TLSVersion.TLSv1_2);self.assertFalse(connection.auto_open)
  args,kw=connection.request.call_args;self.assertEqual(args,('POST','/v1/responses'));self.assertEqual(kw['headers']['Host'],'api.openai.com');self.assertEqual(kw['headers']['Authorization'],'Bearer fixture-key');self.assertEqual(body,b'{}');connection.close.assert_called_once()
 def test_llm_004_redirect_and_auth_bodies_unread(self):
  for status in [301,302,307,308,401,403]:
   (metadata,body),parts=self.send(status,{'Location':'http://127.0.0.1/secret'})
   self.assertEqual(body,b'');self.assertEqual(metadata['httpStatus'],status);parts[-1].read.assert_not_called();self.assertEqual(parts[-2].request.call_count,1)
 def test_openrouter_separate_fixed_origin_and_no_arbitrary_endpoint(self):
  (_,body),(dns,raw,context,connection,reply)=self.send(provider='openrouter-chat')
  dns.assert_called_once_with('openrouter.ai',443,type=socket.SOCK_STREAM);context.wrap_socket.assert_called_once_with(raw,server_hostname='openrouter.ai');self.assertEqual(connection.request.call_args.args,('POST','/api/v1/chat/completions'));self.assertEqual(connection.request.call_args.kwargs['headers']['Host'],'openrouter.ai')
  with patch.object(socket,'getaddrinfo') as dns,self.assertRaises(ValueError):transport.send(b'{}',b'fixture',100,1000,'https://evil.example/')
  dns.assert_not_called()
 def test_llm_022_headers_overflow_and_network(self):
  for headers,data,kind in [({'Content-Type':'text/html'},b'{}',ValueError),({'Content-Encoding':'gzip'},b'{}',ValueError),({'Content-Length':'-1'},b'{}',ValueError),({'Content-Length':'101'},b'{}',OverflowError),({'Retry-After':'yesterday'},b'{}',ValueError),({'Content-Length':'3'},b'{}',ValueError),({'Content-Length':'100'},b'x'*101,OverflowError)]:
   with self.subTest(headers=headers),self.assertRaises(kind):self.send(headers=headers,data=data)
  with self.assertRaises(ConnectionError):self.send(error=ssl.SSLError('synthetic TLS failure'))
  with patch.object(socket,'getaddrinfo',side_effect=socket.gaierror('synthetic DNS failure')),self.assertRaises(socket.gaierror):transport.send(b'{}',b'fixture',100,1000)
if __name__=='__main__':unittest.main(verbosity=2)
