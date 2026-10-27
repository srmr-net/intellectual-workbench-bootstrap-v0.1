"""Internal v1 structural corpus, no modification of canonical public schemas."""
from pathlib import Path
import json,hashlib
from jsonschema import Draft202012Validator
from referencing import Registry,Resource
ROOT=Path(__file__).resolve().parents[3]
def pairs(entries):
 out={}
 for key,value in entries:
  if key in out:raise ValueError('duplicate key')
  out[key]=value
 return out
registry=Registry()
for name in ['composition','room']:
 schema=json.loads((ROOT/f'schemas/{name}.schema.json').read_text());registry=registry.with_resource(schema['$id'],Resource.from_contents(schema))
validator=Draft202012Validator(json.loads((ROOT/'fixtures/vslice-002/snapshot-v1.schema.json').read_text()),registry=registry)
def validate(path):
 value=json.loads(path.read_text(),object_pairs_hook=pairs,parse_constant=lambda _:(_ for _ in ()).throw(ValueError('nonfinite')))
 validator.validate(value)
 digest=value.pop('contentSha256')
 assert digest==hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
 assert value['selectedRoom'] is None or any(r['roomId']==value['selectedRoom'] for r in value['rooms'])
for folder,expected in [('valid',True),('invalid',False)]:
 for path in sorted((ROOT/'fixtures/vslice-002'/folder).glob('*.json')):
  try:validate(path);actual=True
  except Exception:actual=False
  assert actual==expected,path
  print('PASS',folder,path.name)
