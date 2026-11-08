# Architecture decision addendum: additive Atmos 7.1 flat7x row

## Decision

Add one explicit 7.1 flat7x target row alongside the existing 5.1 rows. Do not alter the 5.1 resolver row, its native I0 path, its multirate choices, or its output identity. Do not add automatic fallback from the new row to 5.1.

The existing AD-002 statement that seven profile dimensions are modeled is superseded for this additive-row design. The inspected Rust EncodingProfile has six production fields. ValidationProfile.render_target is not a production dimension. The old 5.1 row therefore remains a six-field production row with validation separate.

The additive 7.1 target uses eight explicit production axes:

1. Authored ADM input profile.
2. I0 independent core-carrier profile.
3. D0 dependent-stream channel-map profile.
4. I0+D0 access-unit topology and order.
5. Per-substream rate allocation.
6. JOC downmix controls (mode and phase controls, separate from reconstruction profile).
7. JOC reconstruction profile (flat7x/index 1).
8. EMDF carrier placement and bounded single-field policy.

Validation render target and observed-vendor-compatible versus ETSI-strict disposition remain in a separate ValidationProfile.

## JOC selector semantics

The current SelectedMode0Phase1Index3 enum combines semantically different data. The 5.1 selected_profile4 receipt records config_mode=0 and phase_flag=1 with carrier_count=5; the separate full-file JOC parse records reconstruction profile index 3/five_x_phase. The official 7.1 structure records reconstruction profile index 1/flat7x with seven carriers and phase_signaling=false.

For the additive row, use a distinct JocReconstructionProfile field. The target value is flat7x/index 1. Do not infer that index 1 means downmix mode 0, phase flag 1, or the JOC wire phase_signaling value. The target downmix-control mode and algorithm phase-flag values remain UNBOUND until separately sourced.

## Architecture boundary

The GTC plan must represent source identity and one generation separately from three role domains: I0 coded channels, D0 chanmap locations, and JOC reconstruction carriers. It must not map by physical row ordinal or assume those counts are interchangeable. F12 produces separate I0 and D0 frame receipts for one source generation; F13 binds one complete metadata container to the designated D0 block; F15 publishes exactly one ordered I0+D0 AU atomically. If either frame, role receipt, metadata bound, or integrity receipt is missing, the AU is rejected without publishing I0 alone.

This addendum approves the profile schema and fail-closed transaction contract. It does not supply the D0 perceptual audio algorithm or the input-specific GTC role mapping. Those remain prerequisites for production output.
