#!/usr/bin/env python3
"""Offline G0 shape fixtures; not runtime/semantic CT conformance."""
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ('component', 'composition', 'ui', 'room')


def load(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f'duplicate JSON key: {key}')
            result[key] = value
        return result
    return json.loads(path.read_text(), object_pairs_hook=unique)


def validators(root=ROOT):
    from jsonschema import Draft202012Validator, FormatChecker
    from referencing import Registry, Resource
    from referencing.exceptions import NoSuchResource

    def offline(uri):
        raise NoSuchResource(ref=uri)

    # Retain the original provisional G0 fixtures against their exact original schema.
    # Canonical ADR-004 UI admission has a separate strict profile/test matrix.
    schemas = {name: load(root / ('schemas/ui-bootstrap.schema.json' if name == 'ui'
                                 else f'schemas/{name}.schema.json')) for name in CONTRACTS}
    registry = Registry(retrieve=offline)
    for schema in schemas.values():
        Draft202012Validator.check_schema(schema)
        registry = registry.with_resource(schema['$id'], Resource.from_contents(schema))
    return {name: Draft202012Validator(schema, registry=registry,
                                      format_checker=FormatChecker())
            for name, schema in schemas.items()}


def check_fixtures(checkers, root=ROOT):
    failures = []
    counts = {(name, kind): 0 for name in CONTRACTS for kind in ('valid', 'invalid')}
    for kind in ('valid', 'invalid'):
        for path in sorted((root / 'fixtures' / kind).glob('*.json')):
            name = path.stem.split('-')[0]
            if name not in checkers:
                failures.append(f'{path.name}: no schema mapping')
                continue
            counts[name, kind] += 1
            errors = list(checkers[name].iter_errors(load(path)))
            if bool(errors) != (kind == 'invalid'):
                detail = errors[0].message if errors else 'negative fixture unexpectedly accepted'
                failures.append(f'{path.relative_to(root)}: {detail}')
    for (name, kind), count in counts.items():
        if count == 0:
            failures.append(f'{name}: missing {kind} fixtures')
    for name, relative in [('component', 'examples/mortgage-calc.wbc/component.json'),
                           ('ui', 'examples/mortgage-calc.wbc/ui.json')]:
        for error in checkers[name].iter_errors(load(root / relative)):
            failures.append(f'{relative}: {error.message}')
    return failures, sum(counts.values())


def main():
    skipped = False
    failures = []
    try:
        checkers = validators()
    except ImportError:
        print('SKIP: install tools/requirements.txt for JSON Schema checks')
        skipped = True
    else:
        failures, count = check_fixtures(checkers)
        print(f'CHECK: {count} fixtures across {len(checkers)} schemas')
    try:
        from rdflib import Graph
    except ImportError:
        print('SKIP: rdflib missing; cannot verify Turtle syntax')
        skipped = True
    else:
        for path in [ROOT / 'ontology/workbench.ttl',
                     ROOT / 'examples/mortgage-calc.wbc/capabilities.ttl']:
            Graph().parse(path, format='turtle')
            print('PASS: RDF parses:', path.relative_to(ROOT))
    for failure in failures:
        print('FAIL:', failure)
    if failures:
        return 1
    if skipped:
        return 2
    print('PASS: all schema fixtures and reference JSON')
    return 0


if __name__ == '__main__':
    sys.exit(main())
