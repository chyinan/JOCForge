# 7.1 per-substream rate allocation: writer boundary

## Scope

This supplement connects the user-approved 7.1 I0/D0 rate allocation to the already sealed static rate receipts. It records a conditional setup mapping and the first writer boundary only. It does not prove that the native DEE setter writes the target frame size or BSI field.

## Bounded static evidence

- The existing rate-pair receipt maps the 384 token to selector code 0 and typed setup value 6; the 1024 token maps to selector code 5 and typed setup value 8 only if the lookup is reached.
- The setup candidate at piVar17+0x206 is not joined to the writer-state owner at param_1+0x810. The writer target at DAT_1806521e8 slot +0x28 and the BSI bitrate field remain UNBOUND.
- A separate sealed carrier-writer receipt leaves the callback cell at zero, the callback target UNBOUND, the carrier frame-size writer UNBOUND, and actual output rate NOT_RUN.

## Target allocation

The authorized 7.1 row is an access-unit aggregate: I0 640 kbps plus D0 1024 kbps. At 48 kHz and six blocks, those per-stream rates correspond to 2,560-byte and 4,096-byte frames; total AU payload is 6,656 bytes for 32 ms. This is not a single 1,664 kbps E-AC-3 frame.

The target I0/D0 allocation is a profile requirement. The DEE static rate receipt does not close the D0 frame-size writer, output BSI rate, D0 audio encoder, or a 7.1 output.

## Handoff status

IMPLEMENTATION_READY=NO for audio production. Keep the old single-stream 5.1 rate row unchanged. The new profile may register this rate allocation as a target contract; production remains gated on the GTC role map, native D0 audio algorithm, D0 writer/BSI boundary, and all-AU EMDF fit. No mode21 output bytes or failure hash are included.
