#!/usr/bin/env python3
"""Trusted worker over private staged files; never run on an untrusted source tree.

The Rust caller bounds this process (CPU, address space, output and wall time).
No retrieval, RDF inference, SPARQL, JS, guest invocation or assurance elevation.
"""
import hashlib
import json
import re
import sys
from pathlib import Path
from urllib.parse import urldefrag
from jsonschema import Draft202012Validator, FormatChecker
from referencing import Registry
from rdflib import Graph, Namespace, RDF, Literal, URIRef
from pyshacl import validate as shacl_validate

ROOT = Path(__file__).resolve().parents[1]
WB = Namespace('https://workbench.example/ns/0.1#')

class Invalid(Exception):
    def __init__(self, code, location):
        self.code, self.location = code, location

def fail(code, location):
    raise Invalid(code, location)

def pairs(items):
    obj = {}
    for key, value in items:
        if key in obj:
            fail('INVALID_MANIFEST', 'component.json')
        obj[key] = value
    return obj

def load(path, code='INVALID_MANIFEST'):
    try:
        return json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=pairs,
                          parse_constant=lambda _: fail(code, path.name))
    except Invalid:
        raise
    except Exception:
        fail(code, path.name)

def path_ref(text):
    if not isinstance(text, str) or not re.fullmatch(r'[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*', text):
        fail('INVALID_PATH', 'component.json')
    if any(p in ('.', '..') for p in text.split('/')) or len(text) > 240:
        fail('INVALID_PATH', 'component.json')
    return text

def validate(stage):
    m = load(stage / 'component.json')
    def dependencies(obj):
        if isinstance(obj, dict):
            for k, v in obj.items():
                if any(s in k.lower() for s in ('dependenc', 'requirespackage', 'packageimports')):
                    fail('UNSUPPORTED_DEPENDENCY', 'component.json')
                dependencies(v)
        elif isinstance(obj, list):
            for v in obj:
                dependencies(v)
    dependencies(m)
    if not isinstance(m, dict):
        fail('INVALID_MANIFEST', 'component.json')
    if m.get('manifestVersion') != '0.1' or (isinstance(m.get('runtime'), dict) and
            m['runtime'].get('interface') != 'org.workbench.component@0.1'):
        fail('UNSUPPORTED_VERSION', 'component.json')
    schema = load(ROOT / 'schemas/component.schema.json')
    validator = Draft202012Validator(schema, format_checker=FormatChecker(), registry=Registry())
    errors = list(validator.iter_errors(m))
    if errors:
        fail('INVALID_MANIFEST', 'component.json')
    # Schema intentionally preserved; tighten its permissive prerelease pattern to SemVer.
    version = m['version'].split('+', 1)[0]
    if '-' in version:
        ids = version.split('-', 1)[1].split('.')
        if any(not s or (s.isdigit() and len(s) > 1 and s[0] == '0') for s in ids):
            fail('INVALID_MANIFEST', 'component.json/version')
    if '+' in m['version'] and any(not s for s in m['version'].split('+')[1].split('.')):
        fail('INVALID_MANIFEST', 'component.json/version')
    if m['permissions']:
        fail('CAPABILITY_DENIED', 'component.json/permissions')
    # UI structure exists; portable ownership/rebinding contract is still provisional.
    if 'ui' in m:
        fail('SCHEMA_VIOLATION', 'component.json/ui')
    used = {'component.json'}
    roles = {'component.json': 'manifest'}
    def use(path, role):
        path_ref(path)
        if path in roles and roles[path] != role:
            fail('INVALID_PATH', 'component.json')
        roles[path] = role
        used.add(path)
        if not (stage / path).is_file():
            fail('MISSING_FILE', path)
        return stage / path
    module = use(m['runtime']['module'], 'module')
    metadata = use(m['metadata'], 'metadata')
    for ref in m.get('evidence', []):
        use(ref, 'evidence')
        load(stage / ref)
    if hashlib.sha256(module.read_bytes()).hexdigest() != m['runtime']['sha256']:
        fail('DIGEST_MISMATCH', m['runtime']['module'])
    schemas, raw = {}, {}
    nodes = 0
    def expand(value, file, stack, depth=0):
        nonlocal nodes
        nodes += 1
        if depth > 64 or nodes > 10000:
            fail('QUOTA_EXCEEDED', file)
        if isinstance(value, list):
            return [expand(v, file, stack, depth+1) for v in value]
        if not isinstance(value, dict):
            return value
        if value.get('$schema', 'https://json-schema.org/draft/2020-12/schema') != 'https://json-schema.org/draft/2020-12/schema':
            fail('SCHEMA_VIOLATION', file)
        if any(k in value for k in ('$dynamicRef', '$recursiveRef', '$dynamicAnchor', '$recursiveAnchor', '$id', '$anchor', '$vocabulary')):
            fail('SCHEMA_VIOLATION', file)
        # Only schema-valued positions are schemas. const/enum/default/examples
        # and property-name maps contain arbitrary JSON and must remain byte-semantic.
        maps = {'$defs', 'definitions', 'properties', 'patternProperties', 'dependentSchemas'}
        arrays = {'allOf', 'anyOf', 'oneOf', 'prefixItems'}
        singles = {'additionalProperties', 'unevaluatedProperties', 'propertyNames',
                   'contains', 'items', 'unevaluatedItems', 'not', 'if', 'then', 'else',
                   'contentSchema'}
        out = {}
        for k, v in value.items():
            if k == '$ref':
                continue
            if k in maps and isinstance(v, dict):
                out[k] = {name: expand(sub, file, stack, depth+1) for name, sub in v.items()}
            elif k in arrays and isinstance(v, list):
                out[k] = [expand(sub, file, stack, depth+1) for sub in v]
            elif k in singles:
                out[k] = expand(v, file, stack, depth+1)
            else:
                out[k] = v
        if '$ref' not in value:
            return out
        ref = value['$ref']
        if not isinstance(ref, str):
            fail('SCHEMA_VIOLATION', file)
        doc, fragment = urldefrag(ref)
        if doc:
            path_ref(doc)  # forbids schemes/URLs/escaping, no network resolver
            target = str(Path(file).parent / doc)
        else:
            target = file
        key = (target, fragment)
        if key in stack:
            fail('SCHEMA_VIOLATION', file)
        if target not in raw:
            use(target, 'schema')
            raw[target] = load(stage / target, 'SCHEMA_VIOLATION')
            try:
                Draft202012Validator.check_schema(raw[target])
            except Exception:
                fail('SCHEMA_VIOLATION', target)
        node = raw[target]
        if fragment:
            if not fragment.startswith('/'):
                fail('SCHEMA_VIOLATION', file)
            try:
                for part in fragment[1:].split('/'):
                    node = node[part.replace('~1', '/').replace('~0', '~')]
            except (KeyError, TypeError):
                fail('SCHEMA_VIOLATION', file)
        resolved = expand(node, target, stack + [key], depth+1)
        if not out:
            return resolved
        out['allOf'] = [resolved, *out.get('allOf', [])]
        return out
    for direction in ('inputs', 'outputs'):
        ports = m['ports'][direction]
        if len({p['name'] for p in ports}) != len(ports):
            fail('INVALID_MANIFEST', 'component.json/ports')
        for p in ports:
            file = p['schema']
            use(file, 'schema')
            raw[file] = load(stage / file, 'SCHEMA_VIOLATION')
            try:
                Draft202012Validator.check_schema(raw[file])
                if isinstance(raw[file], dict) and raw[file].get('$schema', 'https://json-schema.org/draft/2020-12/schema') != 'https://json-schema.org/draft/2020-12/schema':
                    fail('SCHEMA_VIOLATION', file)
                schemas[file] = expand(raw[file], file, [(file, '')])
                Draft202012Validator.check_schema(schemas[file])
            except Invalid:
                raise
            except Exception:
                fail('SCHEMA_VIOLATION', file)
    if {str(p.relative_to(stage)) for p in stage.rglob('*') if p.is_file()} != used:
        fail('INVALID_PATH', 'package')
    g = Graph()
    try:
        g.parse(data=metadata.read_text(encoding='utf-8'), format='turtle', publicID='urn:workbench:staged:')
    except Exception:
        fail('RDF_INVALID', m['metadata'])
    # Unknown Workbench predicates cannot smuggle a competing execution or
    # permission vocabulary. The trusted ontology remains unchanged.
    ontology = Graph().parse(ROOT / 'ontology/workbench.ttl', format='turtle')
    known = set(ontology.subjects(RDF.type, RDF.Property))
    for _, predicate, _ in g:
        if str(predicate).startswith(str(WB)) and predicate not in known:
            fail('RDF_INVALID', m['metadata'] + '#predicate')
        if str(predicate).startswith(str(WB)) and 'permission' in str(predicate).split('#')[-1].lower():
            fail('CAPABILITY_DENIED', m['metadata'] + '#permission')
    # pySHACL shapes are trusted local bytes; data never supplies shapes/imports.
    conforms, _, _ = shacl_validate(g, shacl_graph=str(ROOT / 'crates/wb-semantic/admission-shapes.ttl'),
                                  inference='none', advanced=False, js=False, do_owl_imports=False)
    if not conforms:
        fail('RDF_INVALID', m['metadata'] + '#shape')
    components = set(g.subjects(RDF.type, WB.Component))
    if len(components) != 1:
        fail('SEMANTIC_MISMATCH', m['metadata'] + '#component')
    c = next(iter(components))
    def exact(subject, pred, values):
        if set(g.objects(subject, pred)) != set(values):
            fail('SEMANTIC_MISMATCH', m['metadata'] + '#' + str(pred).split('#')[-1])
    exact(c, WB.componentId, [Literal(m['id'])])
    exact(c, WB.componentVersion, [Literal(m['version'])])
    # No unowned executable/privilege-bearing declarations anywhere in the graph.
    if any(g.triples((None, WB.requestsPermission, None))) or any(g.subjects(RDF.type, WB.PermissionRequest)):
        fail('CAPABILITY_DENIED', m['metadata'] + '#requestsPermission')
    for direction, pred, typ in [('inputs', WB.hasInputPort, WB.InputPort), ('outputs', WB.hasOutputPort, WB.OutputPort)]:
        nodeset = set(g.objects(c, pred))
        if nodeset != set(g.subjects(RDF.type, typ)) or len(nodeset) != len(m['ports'][direction]):
            fail('SEMANTIC_MISMATCH', m['metadata'] + '#ports')
        for p in m['ports'][direction]:
            candidates = [n for n in nodeset if set(g.objects(n, WB.portName)) == {Literal(p['name'])}]
            if len(candidates) != 1:
                fail('SEMANTIC_MISMATCH', m['metadata'] + '#portName')
            n = candidates[0]
            exact(n, WB.schemaRef, [Literal(p['schema'])])
            exact(n, WB.semanticType, [URIRef(p['semanticType'])])
            exact(n, WB.unit, [URIRef(p['unit'])] if 'unit' in p else [])
    arts = set(g.objects(c, WB.hasArtifact))
    if len(arts) != 1 or arts != set(g.subjects(RDF.type, WB.Artifact)):
        fail('SEMANTIC_MISMATCH', m['metadata'] + '#artifact')
    a = next(iter(arts))
    exact(a, WB.artifactPath, [Literal(m['runtime']['module'])])
    exact(a, WB.sha256, [Literal(m['runtime']['sha256'])])
    if any(g.triples((None, WB.hasInterface, None))) or any(g.subjects(RDF.type, WB.AbstractInterface)):
        fail('SEMANTIC_MISMATCH', m['metadata'] + '#ui')
    claims = set(g.objects(c, WB.hasAssuranceClaim))
    if claims != set(g.subjects(RDF.type, WB.AssuranceClaim)):
        fail('SEMANTIC_MISMATCH', m['metadata'] + '#assurance')
    for claim in claims:
        refs = list(g.objects(claim, WB.evidenceRef))
        if len(refs) != 1 or str(refs[0]) not in m.get('evidence', []):
            fail('SEMANTIC_MISMATCH', m['metadata'] + '#evidenceRef')
    capabilities = sorted(str(v) for v in g.objects(c, WB.providesCapability))
    if any(not isinstance(v, URIRef) for v in g.objects(c, WB.providesCapability)):
        fail('SEMANTIC_MISMATCH', m['metadata'] + '#capability')
    return {'manifest': m, 'schemas': schemas, 'capabilities': capabilities, 'trust': 'unreviewed'}

if __name__ == '__main__':
    try:
        print(json.dumps({'ok': validate(Path(sys.argv[1]))}, separators=(',', ':')))
    except Invalid as e:
        print(json.dumps({'error': {'code': e.code, 'location': e.location[:240]}}))
    except Exception:
        print(json.dumps({'error': {'code': 'RDF_INVALID', 'location': 'package'}}))
