# Three-ADM DialNorm cross-input addendum

This addendum extends the previously washed two-input cross-check with a third distinct authorized ADM. Under the same R138-equivalent 5.1 PE profile, 1024 kbps, 1770-4 metering, 48-frame file-position window 2160..2208, DI off and DRC none, the observed pairs were: Human Is Cat (-20.665958 -> U5 21), Hortus Regis Alti (-14.418104 -> U5 14), and My jealousy (-17.093924 -> U5 17).

Under the explicit assumption that each native measured scalar is the approved model's selected_lufs input, the F12 formula predicts the same three U5 values. The native log did not state units; this is a three-input conditional match, not a general rounding/unit rule or measurement-to-BSI owner-generation proof.

Wash independently rehashed all three authorized inputs, the DEE target module, and the custom seven-byte helper source/binary. The helper is a small custom Rust program built against OpenJOC 0.20.1 parser crates, not the official CLI. The user-provided official openjoc.exe is 0.20.1 and its hash was verified, but its inspect reader traverses full declared frames to EOF; it was not invoked under the seven-byte boundary.

No raw prefix, audio payload, PCM, output EC3, license/key contents or paths are included. An earlier capture attempt with zero prefix bytes remains disclosed and is not counted.
