# F06 DialNorm per-source record layout correction

The new Ghidra trace identifies the copied per-source record at +0x30 as a three-pointer dynamic-vector header spanning +0x30..+0x47. Therefore +0x34 is inside the first pointer, not an independent U16 DialNorm field. The BSI serializer's U16 at owner-relative +0x34 belongs to a distinct record.

F06 measurement results are statically copied into per-source record slots, while the separate BSI owner U16 is read by the serializer. No typed same-parent/generation edge joins these different records; the per-source vector's producer and the BSI owner's first writer remain open.

The one fresh R138-equivalent reference still reports measured loudness -33.03 and first-AU DialNorm U5=31, matching only the approved formula's lower-clamp boundary under the assumption that the scalar is selected_lufs. It does not establish native units, rounding, or a measurement update into the BSI owner. An older non-clamped run lacks a process-bound module receipt and was not inspected.

This addendum supersedes the earlier layout ambiguity while preserving the native formula and owner joins as NOT_CLOSED. No source, decompilation, raw media, PCM, payload, or private key is included.
