# Human Cat LFE path: spatial-analysis weight boundary

## Observed path

The authorized ADM metadata maps physical BWF slot 3 / track 4 / `ATU_00000004` to `RoomCentricLFE`. The user also listened to the extracted decoded channel 3 and confirmed it is the low-frequency track. In one matched-config capture, the 38-channel encoder input lane 3 is reached from the corresponding request row. This is an encoder input lane, not proof of the final six-channel E-AC-3 slot mapping.

A separate matched-config capture shows the LFE group-3 mixer doing a unity add. Its first observed non-unity operation is a 4096-point spatial-coding analysis weight: `power[k] = (real[k]^2 + imag[k]^2) * W[k]`. At 48 kHz, bin 2 is 23.4375 Hz and the captured weight is `W[2] = 0.042385533452034`.

This value belongs to a spatial-analysis power path. The capture does not show a sample-amplitude gain applied to the final E-AC-3 LFE waveform, and does not bind the analysis result to the measured channel-3 PCM residual. Do not apply this value as a fixed LFE gain or channel remap. The decoded-core LFE transform remains a separate unresolved path.

## Limits and handling

The adapter and mixer/analysis observations came from separate process invocations; their pointer values are not joined. Neither replay produced an output. The input-only discriminator is symbolic and has no materialized samples or expected output.

The research package disclosed one accidental 8-byte read request at a sample-buffer address while checking a mixer destination pointer. The researcher stopped further sample reads and reports that the bytes were neither retained nor used; this supplement includes no sample values. Treat that as a scope incident, not as measurement evidence.

No PCM, encoded payload, private key, license material, native source code, or disassembly is included.
