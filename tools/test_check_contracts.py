"""Contract fixtures and offline harness regression checks."""
import tempfile
import unittest
from pathlib import Path

import check_contracts as checks


class ContractChecks(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.validators = checks.validators()

    def test_fixture_matrix(self):
        failures, count = checks.check_fixtures(self.validators)
        self.assertGreaterEqual(count, 30)
        self.assertEqual(failures, [])

    def test_duplicate_json_keys_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'duplicate.json'
            path.write_text('{"uiVersion":"0.1","uiVersion":"9"}')
            with self.assertRaisesRegex(ValueError, 'duplicate JSON key'):
                checks.load(path)

    def test_missing_fixtures_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'fixtures/valid').mkdir(parents=True)
            (root / 'fixtures/invalid').mkdir()
            (root / 'examples/mortgage-calc.wbc').mkdir(parents=True)
            for name in ('component', 'ui'):
                (root / f'examples/mortgage-calc.wbc/{name}.json').write_text(
                    (checks.ROOT / f'examples/mortgage-calc.wbc/{name}.json').read_text())
            failures, count = checks.check_fixtures(self.validators, root)
            self.assertEqual(count, 0)
            self.assertEqual(len(failures), 8)

    def test_nested_room_connections_resolve_offline(self):
        room = checks.load(checks.ROOT / 'fixtures/valid/room.json')
        room['connections'] = [{'from': {'instanceId': 'a', 'port': 'out'},
                                'to': {'instanceId': 'b', 'port': 'in'}}]
        self.validators['room'].validate(room)
        del room['connections'][0]['to']['port']
        self.assertFalse(self.validators['room'].is_valid(room))


if __name__ == '__main__':
    unittest.main()
