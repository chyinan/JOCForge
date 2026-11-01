# Two-ADM DialNorm cross-input check

Two distinct authorized ADM BWF inputs were run with the same R138-equivalent PE reference profile, 48-frame file-position window 2160..2208, 1024 kbps, 1770-4 metering, DI off and DRC none. The same target module SHA was joined to each process. Each output parser read exactly seven bytes from the first independent syncframe, zero payload bytes, emitted no raw prefix, and the temporary output and ASCII alias were removed.

Human Is Cat measured -20.665958 and yielded U5 21. Hortus Regis Alti measured -14.418104 and yielded U5 14. Under the explicit assumption that each measured scalar is selected_lufs, the approved model predicts those same U5 values. The native logs did not state units, so the matches remain conditional and limited to these two inputs/configuration.

Parser provenance was independently rechecked: the seven-byte helper source and binary hashes match, and Cargo.lock pins openjoc-eac3/openjoc-bitio 0.20.1. It is a custom helper using those OpenJOC crates, not the user-provided openjoc.exe binary. The official CLI was not used because its inspect path reads complete declared frames to EOF. No output payload was inspected.

This supports using the already approved F12 formula for an MVP cross-input check under the stated assumption. It does not establish a universal DEE rounding/unit rule or the F06 measurement-to-BSI owner generation join. The earlier successful DEE attempt whose file was removed before reading the prefix is disclosed and was not counted as an observation.
