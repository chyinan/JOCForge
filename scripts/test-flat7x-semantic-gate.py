# pattern: Imperative Shell
"""Small regression tests for the Flat-7.X semantic identity gate."""

from __future__ import annotations

import unittest
import importlib.util
from pathlib import Path

_SPEC = importlib.util.spec_from_file_location(
    "flat7x_semantic_audit", Path(__file__).with_name("analyze-flat7x-semantic-audit.py")
)
assert _SPEC is not None and _SPEC.loader is not None
_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)
validate_identity_report = _MODULE.validate_identity_report


def row(input_label: str, dominant_output: str, ratios: list[float]) -> dict[str, object]:
    return {
        "input": input_label,
        "dominant_output": dominant_output,
        "max_rms": 1.0,
        "finite": True,
        "normalized_rms": ratios,
    }


class SemanticGateTests(unittest.TestCase):
    def test_accepts_centre_and_lfe_identity_with_rear_pair_allowance(self) -> None:
        rows = [
            row("L", "FL", [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            row("R", "FR", [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            row("C", "FC", [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            row("Ls", "Ls", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
            row("Rs", "Rs", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            row("Lrs", "Lb", [0.0, 0.0, 0.0, 0.0, 1.0, 0.8, 0.0, 0.0]),
            row("Rrs", "Rb", [0.0, 0.0, 0.0, 0.0, 0.8, 1.0, 0.0, 0.0]),
            row("LFE", "LFE", [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]),
        ]

        self.assertEqual(validate_identity_report(rows, True), [])

    def test_rejects_a_left_to_centre_permutation(self) -> None:
        rows = [row("L", "FC", [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0])]

        failures = validate_identity_report(rows, True)

        self.assertTrue(any("L expected FL" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
