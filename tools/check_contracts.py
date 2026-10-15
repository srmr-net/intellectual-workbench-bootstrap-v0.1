#!/usr/bin/env python3
"""G0 bootstrap smoke checks. Not a full package validator or CT conformance suite."""
from pathlib import Path
import json
import sys
ROOT = Path(__file__).resolve().parents[1]
def main():
    try:
        import jsonschema
    except ImportError:
        print('SKIP: install tools/requirements.txt for JSON Schema checks')
        return 2
    schema=json.loads((ROOT/'schemas/component.schema.json').read_text())
    jsonschema.Draft202012Validator.check_schema(schema)
    validator=jsonschema.Draft202012Validator(schema)
    component=json.loads((ROOT/'examples/mortgage-calc.wbc/component.json').read_text())
    validator.validate(component)
    negative=json.loads((ROOT/'fixtures/invalid/component-missing-version.json').read_text())
    if validator.is_valid(negative): raise AssertionError('negative fixture unexpectedly valid')
    print('PASS: component schema + positive and negative manifest fixtures')
    try:
        from rdflib import Graph
    except ImportError:
        print('SKIP: rdflib missing; cannot verify Turtle syntax')
        return 2
    for p in [ROOT/'ontology/workbench.ttl',ROOT/'examples/mortgage-calc.wbc/capabilities.ttl']:
        Graph().parse(p,format='turtle')
        print('PASS: RDF parses:',p.relative_to(ROOT))
    return 0
if __name__ == '__main__':sys.exit(main())
