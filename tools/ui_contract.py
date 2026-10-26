"""ADR-004 validation over private, already-staged package bytes only."""
import hashlib
import json
import re
from pathlib import Path
from jsonschema import Draft202012Validator
from referencing import Registry
ROOT=Path(__file__).resolve().parents[1]

class UiInvalid(Exception):
    def __init__(self,code):self.code=code

def fail(code):raise UiInvalid(code)
def pairs(entries):
    out={}
    for key,value in entries:
        if key in out:fail('INVALID_UI')
        out[key]=value
    return out

def text(value,max_length):
    return (isinstance(value,str) and bool(value.strip()) and len(value)<=max_length
            and not any(ord(c)<32 or ord(c)==127 or c in '<>{};`' for c in value)
            and not any(p in value.lower() for p in ['://','javascript:','qrc:','qt.','=>','import qt']))

def field_name(pointer):
    if not isinstance(pointer,str) or not pointer.startswith('/') or '/' in pointer[1:]:fail('UI_BINDING_MISMATCH')
    if re.search(r'~(?![01])',pointer):fail('UI_BINDING_MISMATCH')
    result=pointer[1:].replace('~1','/').replace('~0','~')
    if result in ('__proto__','prototype','constructor'):fail('UI_BINDING_MISMATCH')
    return result

def validate(raw,manifest,schemas,path):
    if len(raw)>65536:fail('UI_QUOTA_EXCEEDED')
    try:document=json.loads(raw.decode('utf-8'),object_pairs_hook=pairs,parse_constant=lambda _:fail('INVALID_UI'))
    except UiInvalid:raise
    except Exception:fail('INVALID_UI')
    if not isinstance(document,dict) or document.get('uiVersion')!='0.1':fail('UNSUPPORTED_UI_VERSION')
    schema_bytes=(ROOT/'schemas/ui.schema.json').read_bytes()
    checker=Draft202012Validator(json.loads(schema_bytes),registry=Registry())
    if not checker.is_valid(document):fail('INVALID_UI')
    if not text(document['root']['title'],160):fail('UNSAFE_UI_CONTENT')
    identifiers=set();used=set();input_port=None
    for c in document['root']['children']:
        if c['id'] in identifiers:fail('UI_DUPLICATE_ID')
        identifiers.add(c['id'])
        if not text(c['label'],160) or ('unit' in c and not text(c['unit'],40)):fail('UNSAFE_UI_CONTENT')
        direction='outputs' if c['type']=='readout' else 'inputs'
        if c['bind']['direction']!=('output' if direction=='outputs' else 'input'):fail('UI_BINDING_MISMATCH')
        port=next((p for p in manifest['ports'][direction] if p['name']==c['bind']['port']),None)
        if port is None:fail('UI_BINDING_MISMATCH')
        contract=schemas[port['schema']]
        field=field_name(c['bind']['field'])
        if not isinstance(contract,dict) or contract.get('type')!='object':fail('UI_BINDING_MISMATCH')
        prop=contract.get('properties',{}).get(field)
        if not isinstance(prop,dict):fail('UI_BINDING_MISMATCH')
        kind=prop.get('type')
        if c['type']=='decimal-input' and (kind!='string' or not isinstance(prop.get('pattern'),str)):fail('UI_BINDING_MISMATCH')
        if c['type']=='integer-input' and kind!='integer':fail('UI_BINDING_MISMATCH')
        if c['type']=='readout' and kind not in ('string','integer'):fail('UI_BINDING_MISMATCH')
        if direction=='inputs':
            if input_port not in (None,port['name']) or field in used:fail('UI_BINDING_MISMATCH')
            input_port=port['name'];used.add(field)
    if input_port is None:fail('UI_BINDING_MISMATCH')
    port=next(p for p in manifest['ports']['inputs'] if p['name']==input_port)
    if not set(schemas[port['schema']].get('required',[])).issubset(used):fail('UI_BINDING_MISMATCH')
    return {'path':path,'sha256':hashlib.sha256(raw).hexdigest(),'schema_sha256':hashlib.sha256(schema_bytes).hexdigest(),'document':document}
