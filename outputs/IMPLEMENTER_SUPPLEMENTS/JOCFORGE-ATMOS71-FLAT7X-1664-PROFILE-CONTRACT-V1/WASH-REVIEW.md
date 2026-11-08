# Wash review: additive 7.1 flat7x / 1664 kbps profile

## Disposition

The user has authorized an additive 7.1 flat7x target while retaining the existing native 5.1 multirate path. This package records the target profile contract and the GTC ownership shape for implementation planning. It does not approve a production D0 audio encoder or claim that JOCForge has generated a 7.1 output.

The implementation scope is limited to registering the profile contract, modeling the typed per-substream access-unit plan, and adding fail-closed profile admission. Production codec output remains blocked until the D0 source-role projection and native D0 audio path are closed with Wash evidence. No audio algorithm may be inferred from the old main branch, the mode21 failed carrier, FFmpeg, or another encoder.

## Independent review

- The supplied draft ZIP digest matches. Its package verifier passes normally and under python -O. The draft remains a draft and is not forwarded.
- Its 29 current-source file receipts all match the live isolated worktree. The recorded HEAD is stale: the live branch is at a7b7aaad5d5b21babe06b57f660958c7127454c5, while origin/experimental/independent-encoder is at 01c9ce35dc0b5dacb617a4c338af6dd6418a443d. The worktree is dirty. Wash changed none of it.
- The hash-bound official structure summary reports I0+D0, 7.1, D0 chanmap 0x1A00, flat7x/profile index 1, seven non-LFE reconstruction carriers, 15 objects/rows, and complexity 16. The official sample uses I0=640 and D0=640 kbps; the requested D0=1024 rate is a separate rate-allocation requirement.
- The prior mode21 summary reports I0=640 and D0=1024 kbps, but its EMDF carriage is explicitly excluded. No mode21 output bytes or hash are included in this supplement.
- ETSI TS 102 366 Table E.1.4 maps chanmap bits 3, 4, and 6 in 0x1A00 to Left Surround, Right Surround, and the Left/Right Rear Surround pair. This yields four D0 coded channels, consistent with acmod 5 (3/1) and lfeon=0. These are codec map locations, not ADM physical-row count or JOC carrier count.
- The current Rust EncodingProfile has six production fields; ValidationProfile.render_target is separate. The architecture text's generic seven-dimension count is inconsistent with that structure. This addendum defines the additive target with eight explicit production axes and keeps validation profile external.
- The existing Mode0/Phase1/Index3 variant is composite. The selected 5.1 parameter receipt has config_mode=0 and phase_flag=1; a separate 5.1 full-file parse reports profile index 3/five_x_phase. The official 7.1 structure reports profile index 1/flat7x and phase_signaling=false. Treat index3/five_x_phase and index1/flat7x as values of a separate JOC reconstruction-profile selector. Do not equate that selector with mode0, phase_flag, or the wire phase_signaling bit. For the target row, mode/phase control values remain UNBOUND.

## Open production gates

1. The exact selected ADM input and its GTC owner/generation projection to I0, D0, and JOC roles remain UNBOUND. N expansion stays deferred; use only an already accepted 36/38-channel input when a later run is authorized.
2. No Wash-approved native D0 audio coding algorithm or typed D0 source-role transfer is present in this package.
3. The required D0 block-2 single-skip-field EMDF contract is a user-approved target and matches the observed official carrier location at D0 skipfld level. The current sanitized official summary does not independently expose the exact block ordinal; the next output acceptance must assert block 2 directly.
4. The maximum complete EMDF size of 511 bytes follows the 9-bit skipl field. The new 7.1 payload size distribution is NOT_RUN. Any container over 511 bytes must fail closed before any AU is published; no cross-field continuation is allowed.
5. Official-sample validator dispositions are separate: observed-vendor-compatible accepts the reference; ETSI-strict rejects it because payload IDs 11/14 use codecdatae=0. This is not a claim of strict ETSI conformance for a future output.

## Approved profile row

The machine-readable target row is in PROFILE-ROW.json. The row is approved as an architecture/profile contract only. Production D0 encoding is not approved by this package.
