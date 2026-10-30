# DDP local slot to frame channel index; ADM/global join remains open

Disposition: PARTIAL_STATIC_BOUNDARY_EVIDENCE. This reconciles DDP adapter v7 with the earlier selector reference. It does not establish the runtime branch for the reference call, connect DDP to ADM identity, or close U04/Gate1.

Source DDP package manifest SHA-256: 1ff077f7e348212df989762d34928da28546473ec022530afcf2dd5072d1c27e.
Source EVIDENCE.json SHA-256: 32e5a0de13d3a9ab7942f25a09c3becc0b45346d174a4289297f1111eb57cbc6.
Superseded v5 manifest: 180a4b561b2e8e4326d30fdda5bd2f3b7d0fc28a415df05c84255c54471f273b.
Prior selector package manifest: 55ec6020b9b5f5f5802a359febaf37498592621b8b026e9a407d67e0d121892f; EVIDENCE hash: f6d84b07d4071f59216bf3111f704f53fbab8d48432b68f5801c550b82cb201c.
Both source checkers passed normal and optimized Python. Wash rechecked listed member hashes and supersession references. Wash did not independently reproduce the static trace.

## New DDP edge reported by v7

On the static non-mode-1 config branch, one u16 count at channel-config offset 10 is used for both six-byte config-row copying and PCM-channel copying. The adapter copies input pointer-table entry i to internal channel-buffer slot i, preserving the ordinal. The DDP descriptor-row builder writes its zero-based record ordinal into the local pointer-table slot; the dispatcher indexes that table by the slot. This supports local slot i to frame input channel slot i for that static branch.

The branch and values for the reference call were NOT_CAPTURED. The top-level 96-byte per-source record count is separate; equality with selector width 48 is NOT_PROVEN. Frame input channel index is not mapped to CHNA UID, CHNA track index, physical slot, or global source ID.

## Prior selector identity in a separate reference invocation

The prior selector package used the same authorized 38-channel BWF hash 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1 and approved F03R crosswalk hash dda4c052c11b47e5f2d4245ae3d9e9736ebb0dc1fb793ebdb5bd0b4cbf7bbac2. In its completed reference invocation, 48 descriptor columns had an 11-column aggregate prefix and 37 regular owner rows; row i mapped to column i+11, columns 11..47 mapped to global IDs 11..47. It observed 36 candidate-gate rows, 15 selected rows, and a 15x48 consumer.

That reference job XML was befb67c91ae67489090062d287785121c07bd34db0985d9b0dbed70279376b9e, not current accepted XML 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9. This is same-input reference evidence, not a current-job run. Numeric matrix coefficients and U04 invocation/output were NOT_CAPTURED.

The converter reports AO_xxxx minus 0x1001 and ATU_xxxxxxxx minus 1; it does not prove ATU equals a CHNA track index or join it to the DDP channel index.

## Remaining

DDP frame channel index to ADM UID/CHNA physical slot/global ID is UNBOUND. DDP frame channel index to selector columns 11..47 is UNBOUND. Same-generation DDP runtime join is NOT_CAPTURED. Latest short replay exited before measurement and produced no output. Gate1, U04 numeric replay, Type10, and six-plane checks remain NOT_RUN.

Do not join these domains by ordinal, count, or shape. Next trace the DDP frame descriptor producer's input pointer table and six-byte config rows to an explicit ADM UID/CHNA track-index receipt, then bind that channel to the selector table and approved F03R crosswalk. No source code, decompilation, addresses, raw payload, license, or key is included.
