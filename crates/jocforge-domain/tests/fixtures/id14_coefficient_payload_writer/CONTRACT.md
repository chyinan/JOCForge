# ID14 coefficient payload writer: selected controlled slice

This internal supplement closes the selected native coefficient-input → quantization → JOC coefficient bitstream → ID14 payload writer → descriptor-copy slice for controlled self-owned synthetic non-audio vectors. It does not close the upstream audio/fit producer or whole F13C.

## Inputs and writer contract

`INPUT-ONLY.json` contains 15 self-owned synthetic coefficient cases; none contains PCM or a native reference body as an encoder input. `approved-codebooks.json` contains six normal non-secret Huffman/code tables (full/sparse, coarse/fine, and 5-/7-channel position tables) totaling 588 symbols, with the approved companion provenance retained in the file. The selected zero case is downmix 3/five channels, 15 objects, band enum 5, full-fine mode, one point, clip 128, sequence 0 and caller capacity 372 bytes. It writes a 150-bit descriptor/header plus 900 coefficient bits, 1050 bits total, padded to 132 bytes.

The coefficient input is already stored in the native F32×1/16 domain. Quantization applies one F32 multiplication by 32768, lrintf-style nearest-even under the tested process environment, signed-16 clamp, sign-dependent bias, division toward zero, center add and output clamp. Fine mode uses center96/bias102/divisor205/range0..191; coarse uses center48/bias205/divisor410/range0..95. Positive and negative zero map to the center. Full mode resets previous to48/96 per channel and point; sparse mode writes a selected channel and position deltas, then resets coefficient previous to50/100.

All symbols and widths for 588 native Huffman entries match the approved tables. Descriptor/header fields precede the coefficient body; bits are MSB-first with no per-object byte alignment; final padding is zero. Successful native payloads are copied by the same call into a distinct owned ID14 descriptor cache; the test verifies descriptor ID, length, body equality and cache-pointer preservation.

## Native differences and limits

This is a controlled writer-slice result, not end-to-end audio. The 15 cases inject synthetic coefficients at the native coefficient tables; they do not run upstream QMF, target fitting, normalization or audio scoring. No production owner/cold generation, transport, authentication, AU publication or file commit is claimed.

Native clip is produced from a peak/hold state and threshold/compensation path. The frozen model obtains clip from target-gain metadata. This supplement accepts clip as an explicit 8-bit input and does not close that producer difference. `time_slot_limit=24` is an offset-validity threshold, not a channel count. Native offset24 can return writer success with validity false; offset32 returns3 after writing a prefix; equal two offsets return0 with validity false. The selected one-point case is unaffected. The parent checks writer return, not this validity flag. Native writer failures can retain a written prefix; the approved zero-commit candidate remains the clean implementation policy, not a native rollback claim.

The native representation-degradation retry helper is separate from the payload writer; its loop ignores a no-progress result. The washed model's finite retry limit/minimum-error rule is an explicit safe policy and must not be described as native behavior. No private key, authentication value, raw PCM, DLL, or decompiled source is included. Whole F13C and product acceptance remain NOT_READY.
