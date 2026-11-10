# Wash handoff: Flat7x partial structure profile

## Delivery

Package ID: `JOCFORGE-FLAT7X-1024-PARTIAL-STRUCTURE-V1`  
Status: `PARTIAL_IMPLEMENTATION_READY`

This handoff authorizes the implementation agent to begin the user-requested partial Flat7x profile work. It covers the new typed reconstruction-layout selection and its normative JOC header field on the existing 1024 kbps / 5.1 core path.

## Confirmed contract

- `joc_dmx_config_idx` is 3 bits in the ETSI `joc_header()` sequence, before object-count and extension-index fields.
- Index 1 maps to 7.X `Lb/Rb`, with seven JOC downmix channels per Table 48.
- The current internal mode0/phase1/index3 profile remains a separate control value and keeps existing behavior.
- `joc_num_objects_bits` and `complexity_index_type_a` remain separate fields; the current slice does not assign or derive either value.
- Existing rate, core audio algorithm, input resolution, stream topology, and publication behavior are inherited from the approved 1024 kbps / 5.1 baseline.

## GTC and owner

The reconstruction layout is immutable attempt configuration. It passes through the existing profile-resolution value to the JOC header writer. This partial change adds no mutable state and no new commit owner.

## Remaining gates

The complete official-demo bitstream has not been captured or parsed by Wash. The native selector route, remaining JOC fields, outer AU structure, decoded 7.1 rendering, and listening acceptance remain independent gates. The implementation agent can proceed with this bounded slice and report it as partial.
