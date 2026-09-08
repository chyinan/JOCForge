# pattern: Imperative Shell
"""Regression tests for the captured pre-core channel-order contract."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


_SPEC = importlib.util.spec_from_file_location(
    "source_carrier_correlation", Path(__file__).with_name("analyze-source-carrier-correlation.py")
)
assert _SPEC is not None and _SPEC.loader is not None
_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)
pre_core_channel_labels = _MODULE.pre_core_channel_labels


class PreCoreOrderTests(unittest.TestCase):
    def test_current_ffmpeg_raw_order_puts_lfe_before_surrounds(self) -> None:
        self.assertEqual(
            pre_core_channel_labels("ffmpeg-5.1"),
            ("L", "R", "C", "LFE", "Ls", "Rs"),
        )

    def test_legacy_capture_order_remains_explicit(self) -> None:
        self.assertEqual(
            pre_core_channel_labels("jocforge-semantic-6"),
            ("L", "R", "C", "Ls", "Rs", "LFE"),
        )


if __name__ == "__main__":
    unittest.main()
