"""Exercise source membership and exact fixture identity independently."""
import hashlib
from pathlib import Path
import runpy
import tempfile
import unittest


checker = runpy.run_path(str(Path(__file__).with_name("check-boundaries.py")))


class InventoryChecks(unittest.TestCase):
    def test_source_can_change_but_must_exist(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "parser.rs"
            record = {"path": source.name}
            for text in ["fn parse() {}", "// new implementation\nfn parse() {}"]:
                source.write_text(text, encoding="utf-8")
                checker["check_inventory_file"](root, record)
            source.unlink()
            with self.assertRaises(checker["BoundaryError"]):
                checker["check_inventory_file"](root, record)

    def test_same_size_fixture_mutation_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative in ["vector.bin", "crates/audio/tests/fixtures/recipe.py"]:
                fixture = root / relative
                fixture.parent.mkdir(parents=True, exist_ok=True)
                fixture.write_bytes(b"abc")
                record = {"file": relative, "bytes": 3, "sha256": hashlib.sha256(b"abc").hexdigest()}
                checker["check_inventory_file"](root, record)
                fixture.write_bytes(b"abd")
                with self.assertRaises(checker["BoundaryError"]):
                    checker["check_inventory_file"](root, record)


if __name__ == "__main__":
    unittest.main()
