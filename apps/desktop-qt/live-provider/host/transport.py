"""Trusted one-shot HTTPS transport. Fixed origin, no tools/proxies/redirects/log bodies."""
import http.client,ipaddress,json,socket,ssl,struct,sys,time
HOST='api.openai.com'
PATH='/v1/responses'
ORIGINS={'openai-responses':(HOST,PATH),'openrouter-chat':('openrouter.ai','/api/v1/chat/completions')}
http.client._MAXLINE=4096
http.client._MAXHEADERS=32

def public_addresses(records):
 result=[]
 for family,kind,proto,canon,address in records:
  ip=ipaddress.ip_address(address[0]);mapped=getattr(ip,'ipv4_mapped',None)
  candidates=[ip]+([mapped] if mapped else [])
  if family not in (socket.AF_INET,socket.AF_INET6) or kind!=socket.SOCK_STREAM or address[1]!=443 or any(not x.is_global or x.is_multicast or x.is_reserved or x.is_loopback or x.is_link_local or x.is_unspecified for x in candidates):raise ValueError('DESTINATION')
  if address not in [a for _,a in result]:result.append((family,address))
 if not result or len(result)>8:raise ValueError('DESTINATION')
 return result

def send(body,key,max_bytes,timeout_ms,provider="openai-responses"):
 if provider not in ORIGINS:raise ValueError("PROVIDER")
 host,path=ORIGINS[provider]
 deadline=time.monotonic()+min(timeout_ms,30000)/1000
 addresses=public_addresses(socket.getaddrinfo(host,443,type=socket.SOCK_STREAM))
 context=ssl.create_default_context();context.minimum_version=ssl.TLSVersion.TLSv1_2
 connected=None
 for family,address in addresses:
  raw=socket.socket(family,socket.SOCK_STREAM)
  try:
   left=deadline-time.monotonic()
   if left<=0:raise TimeoutError()
   raw.settimeout(min(2,left));raw.connect(address)
   raw.settimeout(max(.001,deadline-time.monotonic()))
   connected=context.wrap_socket(raw,server_hostname=host);break
  except (OSError,ssl.SSLError):raw.close()
 if connected is None:raise ConnectionError()
 connection=http.client.HTTPSConnection(host,443,context=context);connection.sock=connected;connection.auto_open=False
 try:
  connection.request('POST',path,body=body,headers={'Host':host,'Authorization':'Bearer '+key.decode('ascii'),'Content-Type':'application/json','Accept':'application/json','Accept-Encoding':'identity','Connection':'close'})
  reply=connection.getresponse()
  # Never follow Location. No response header or message is copied into diagnostics.
  status=reply.status
  if status in (401,403) or 300<=status<400:return {'httpStatus':status,'failure':None,'retryAfterMs':None,'bodyLength':0},b''
  if reply.getheader('Content-Encoding','identity').lower() not in ('','identity'):raise ValueError('HEADER')
  if reply.getheader('Content-Type','').split(';',1)[0].strip().lower()!='application/json':raise ValueError('HEADER')
  length=reply.getheader('Content-Length')
  if length is not None and (not length.isascii() or not length.isdecimal() or len(length)>12):raise ValueError('HEADER')
  if length is not None and int(length)>max_bytes:raise OverflowError()
  retry=reply.getheader('Retry-After');retry_ms=None
  if retry is not None:
   if not retry.isascii() or not retry.isdecimal() or len(retry)>6:raise ValueError('HEADER')
   retry_ms=int(retry)*1000
  data=reply.read(max_bytes+1)
  if len(data)>max_bytes:raise OverflowError()
  if length is not None and len(data)!=int(length):raise ValueError('HEADER')
  return {'httpStatus':status,'failure':None,'retryAfterMs':retry_ms,'bodyLength':len(data)},data
 finally:connection.close()

def main():
 key=sys.stdin.buffer.readline(4098)
 if not key.endswith(b'\n') or not 2<=len(key)<=4097:raise ValueError('CREDENTIAL')
 key=key[:-1]
 if not all(33<=b<=126 for b in key):raise ValueError('CREDENTIAL')
 prefix=sys.stdin.buffer.read(4)
 if len(prefix)!=4:raise ValueError('FRAME')
 size=struct.unpack('>I',prefix)[0]
 if not 1<=size<=66000:raise ValueError('FRAME')
 raw=sys.stdin.buffer.read(size)
 if len(raw)!=size or sys.stdin.buffer.read(1):raise ValueError('FRAME')
 request=json.loads(raw)
 if set(request)!={'provider','body','maxResponseBytes','timeoutMs'} or not isinstance(request['body'],str) or not 1<=request['maxResponseBytes']<=32768 or not 1<=request['timeoutMs']<=30000:raise ValueError('FRAME')
 return send(request['body'].encode('utf-8'),key,request['maxResponseBytes'],request['timeoutMs'],request['provider'])
if __name__=='__main__':
 try:metadata,body=main()
 except (socket.timeout,TimeoutError):metadata,body={'httpStatus':0,'failure':'TIMEOUT','retryAfterMs':None,'bodyLength':0},b''
 except OverflowError:metadata,body={'httpStatus':0,'failure':'BUDGET_EXCEEDED','retryAfterMs':None,'bodyLength':0},b''
 except ValueError:metadata,body={'httpStatus':0,'failure':'MALFORMED_RESPONSE','retryAfterMs':None,'bodyLength':0},b''
 except Exception:metadata,body={'httpStatus':0,'failure':'UNAVAILABLE','retryAfterMs':None,'bodyLength':0},b''
 encoded=json.dumps(metadata,separators=(',',':')).encode();sys.stdout.buffer.write(struct.pack('>I',len(encoded))+encoded+body);sys.stdout.buffer.flush()
