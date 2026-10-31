# Scoped measurement and BSI writer observation

## Disposition

This sanitized note is for Wash review. It contains no local paths, execution XML, raw addresses, decompilation, binaries, PCM, or bitstream payload.

Ghidra found a static U16-to-five-bit writer candidate, but its field meaning and owner/generation remain **UNBOUND**. In a completed run on one authorized 38-channel ADM BWF, the candidate's exact source and width-five callsites were not observed. A separate bounded capture observed a generic width-five helper writing scalar `1`; its return site did not match the static candidate. That scalar is not identified as DialNorm.

The exact candidate edge is not demonstrated as a gate to process completion for this sample. The run does not establish DialNorm correctness, output conformance, or spatial playback.

## Coverage

- One real 38-channel ADM BWF was used with the Atmos DDP filter at 1024 kbps.
- The runs exited and produced temporary output files; only file existence and byte count were observed. The outputs were removed without reading or retaining payload.
- A fresh run repeated the log scalars `-16.784912` and `-16.78`; the log states no units and neither scalar is joined to a typed receipt or BSI field.
- The process-level `TEMP`/`TMP` class was system default in the successful WinDbg runs. Earlier attempts with overridden `TEMP`/`TMP` failed plugin initialization; the cause was not isolated.
- 48 kHz output, 5.1 configuration, JOC signaling, a second real ADM sample, and player playback were not verified here.

## Open edge

The measurement finalizer to F06ND DRC input and F06ND to F12 BSI DialNorm joins remain **UNBOUND**. The next bounded edge is the actual caller and typed BSI field behind the observed generic width-five helper. Matching a scalar value or bit width is not an owner or generation join.
