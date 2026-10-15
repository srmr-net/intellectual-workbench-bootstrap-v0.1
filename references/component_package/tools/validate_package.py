#!/usr/bin/env python3
"""Validate a local Workbench v0.1 package; requires jsonschema and rdflib."""
import hashlib, json, sys
from pathlib import Path
from jsonschema import Draft202012Validator, FormatChecker
from rdflib import Graph, Namespace, RDF, URIRef, Literal
WB=Namespace('https://workbench.example/ns/0.1#')

def validate(package: Path, schema: Path):
    errors=[]
    try:
        m=json.loads((package/'component.json').read_text())
        definition=json.loads(schema.read_text())
        Draft202012Validator.check_schema(definition)
        for err in Draft202012Validator(definition,format_checker=FormatChecker()).iter_errors(m): errors.append('manifest: '+err.message)
    except Exception as e: return ['manifest: '+str(e)]
    if errors: return errors
    paths=[m['runtime']['module'],m['metadata'],*( [m['ui']] if 'ui' in m else []),*m.get('evidence',[])]
    for d in ['inputs','outputs']:
        paths.extend(p['schema'] for p in m['ports'][d])
    for p in paths:
        loc=(package/p).resolve()
        if not loc.is_relative_to(package.resolve()): errors.append('path escapes package: '+p); continue
        if loc.is_symlink(): errors.append('symlink forbidden: '+p)
        if not loc.is_file(): errors.append('missing file: '+p)
    if (package/m['runtime']['module']).is_file():
        digest=hashlib.sha256((package/m['runtime']['module']).read_bytes()).hexdigest()
        if digest!=m['runtime']['sha256']: errors.append('runtime sha256 mismatch')
    for direction in ['inputs','outputs']:
        names=[p['name'] for p in m['ports'][direction]]
        if len(names)!=len(set(names)): errors.append('duplicate '+direction+' port names')
        for p in m['ports'][direction]:
            try:
                s=json.loads((package/p['schema']).read_text()); Draft202012Validator.check_schema(s)
            except Exception as e: errors.append('invalid port schema '+p['schema']+': '+str(e))
    try:
        g=Graph();g.parse(package/m['metadata'],format='turtle')
        comps=list(g.subjects(RDF.type,WB.Component))
        matches=[c for c in comps if (c,WB.componentId,Literal(m['id'])) in g and (c,WB.componentVersion,Literal(m['version'])) in g]
        if len(matches)!=1: errors.append('RDF component id/version mismatch or duplicate')
        else:
            c=matches[0]
            for direction,predicate,typ in [('inputs',WB.hasInputPort,WB.InputPort),('outputs',WB.hasOutputPort,WB.OutputPort)]:
                rdfports=list(g.objects(c,predicate)); decl=m['ports'][direction]
                if len(rdfports)!=len(decl): errors.append('RDF '+direction+' count mismatch')
                for p in decl:
                    cand=[r for r in rdfports if (r,WB.portName,Literal(p['name'])) in g and (r,RDF.type,typ) in g]
                    if len(cand)!=1: errors.append('RDF port missing/ambiguous: '+p['name']);continue
                    r=cand[0]
                    if (r,WB.semanticType,URIRef(p['semanticType'])) not in g: errors.append('RDF semantic type mismatch: '+p['name'])
                    if (r,WB.schemaRef,Literal(p['schema'])) not in g: errors.append('RDF schemaRef mismatch: '+p['name'])
                    units=list(g.objects(r,WB.unit))
                    if (str(units[0]) if len(units)==1 else None)!=p.get('unit'): errors.append('RDF unit mismatch: '+p['name'])
            art=list(g.objects(c,WB.hasArtifact))
            if len(art)!=1 or (art[0],WB.artifactPath,Literal(m['runtime']['module'])) not in g or (art[0],WB.sha256,Literal(m['runtime']['sha256'])) not in g: errors.append('RDF artifact path/digest mismatch')
            uis=list(g.objects(c,WB.hasInterface))
            if 'ui' in m:
                if len(uis)!=1 or (uis[0],WB.uiDefinitionRef,Literal(m['ui'])) not in g: errors.append('RDF ui reference mismatch')
            elif uis: errors.append('RDF declares UI missing in manifest')
            if list(g.objects(c,WB.requestsPermission)) and not m['permissions']: errors.append('RDF permission request absent from manifest')
    except Exception as e: errors.append('RDF parsing/check: '+str(e))
    return errors

if __name__=='__main__':
    if len(sys.argv)!=3: sys.exit('usage: validate_package.py PACKAGE_DIR MANIFEST_SCHEMA')
    issues=validate(Path(sys.argv[1]),Path(sys.argv[2]))
    for issue in issues: print('ERROR:',issue)
    print('VALID' if not issues else f'INVALID ({len(issues)} errors)')
    sys.exit(bool(issues))
