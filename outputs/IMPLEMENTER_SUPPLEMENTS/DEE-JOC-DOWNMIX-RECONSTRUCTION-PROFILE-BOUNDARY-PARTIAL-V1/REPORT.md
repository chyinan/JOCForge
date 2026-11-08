# JOC downmix-control and reconstruction-profile boundary

## Scope

This supplement cross-checks hash-bound 5.1 parameter and JOC parse receipts against the official 7.1 structure summary. It resolves the semantic distinction between downmix/phase controls and the JOC reconstruction-profile selector, but it does not identify the serializer that connects the legacy Rust enum to either wire field.

## Bounded findings

- The approved 5.1 selected_profile4 receipt records config_mode=0, phase_flag=1, and carrier_count=5. A separate F05J contract derives a three-bit downmix_index value of 3 for that 5.1 control combination.
- The independent full-file 5.1 JOC parse reports profile_index=3 and five_x_phase.
- The official 7.1 structure summary reports profile_index=1, flat7x, seven carriers, and phase_signaling=false.
- Therefore index 3/five_x_phase and index 1/flat7x are two values of the reconstruction-profile field. config_mode and phase_flag are separate algorithm controls. The wire phase_signaling bit is not proven identical to the algorithm phase_flag.
- Equal numeric value 3 does not join the 5.1 downmix_index to the 5.1 reconstruction profile_index. The legacy SelectedMode0Phase1Index3 enum is composite, but its Index3 component's typed serializer mapping remains UNBOUND.
- The DDP XML preferred_downmix_mode path is a separate MixMode property; its observed static setter/parser path does not establish a JOC reconstruction-profile consumer.

## Handoff

IMPLEMENTATION_READY=NO for the serializer edge. The additive 7.1 target may use the observed flat7x/index1 reconstruction profile, but its config_mode and algorithm phase control remain UNBOUND. Do not infer these values from the old enum name or from phase_signaling=false.
