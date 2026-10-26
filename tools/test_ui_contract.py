"""Strict new UI fixture corpus; original bootstrap tests remain unchanged."""
import json
import unittest
from pathlib import Path
from ui_contract import validate, UiInvalid
ROOT = Path(__file__).resolve().parents[1]

class UiFixtures(unittest.TestCase):
    def test_positive_and_malicious_fixtures(self):
        package = ROOT / 'target/mortgage-packages/mortgage-headless'
        manifest = json.loads((package / 'component.json').read_text())
        schemas = {p['schema']: json.loads((package / p['schema']).read_text())
                   for direction in ('inputs', 'outputs') for p in manifest['ports'][direction]}
        for p in (ROOT / 'fixtures/ui/valid').glob('*.json'):
            with self.subTest(path=p.name):
                self.assertEqual(validate(p.read_bytes(), manifest, schemas, 'ui.json')['document']['uiVersion'], '0.1')
        for p in (ROOT / 'fixtures/ui/invalid').glob('*.json'):
            with self.subTest(path=p.name), self.assertRaises(UiInvalid):
                validate(p.read_bytes(), manifest, schemas, 'ui.json')
