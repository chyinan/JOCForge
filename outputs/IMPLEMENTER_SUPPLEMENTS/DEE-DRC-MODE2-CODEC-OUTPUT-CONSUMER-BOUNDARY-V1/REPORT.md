# Mode-2 DRC codec output-consumer boundary

## Disposition

MODE2_CODEC_INPUT_TO_OUTPUT_DESCRIPTOR_CONSUMER_BOUND_STATIC; LINE_A0_TRANSFER_UNBOUND; OUTPUT_SLOT_TARGET_UNBOUND; BSI_AND_AUDIO_EFFECT_UNBOUND; RUNTIME_NOT_RUN.

The new Ghidra slice follows the previously bound mode-2 codec entry through state preparation to the first output-descriptor consumer. It does not bind line record +0xa0, an equivalent profile parameter, or a BSI effect.

## Bound path

For the selected mode-2 condition, FUN_180179ab0 constructs codec configuration from block-descriptor fields +0x18/+0x20 and implementation state +0x88, then FUN_1801a8840 passes it to FUN_1801ac850. FUN_1801a7710 proceeds to FUN_1801ab800, which constructs an output descriptor from the output buffer and byte count and passes it to FUN_1801b1590.

FUN_1801b1590 selects a per-item object from context array +0xc0f8 and dispatches by an object-kind table. The output-side slot +0x28 callsite is 0x1801b18d9, but the runtime kind and typed target are not bound. The inspected codec and descriptor arguments contain no explicit record pointer or record +0xa0 field. No typed equivalent line-profile parameter was established in the inspected calls.

## Limits and verification

Wash independently recomputed the source evidence manifest and package seal, all seven seal entries, eleven source evidence hashes, parent receipt hashes, and the target DLL hash. The source agent reports normal and python -O checker PASS; Wash did not execute the checker.

No runtime selection, DEE run, media/payload/key/license read, or implementation contact is claimed. The evidence is scoped to this mode-2 path and makes no global absence claim.

## Implementation boundary

Treat the mode-2 route to an output descriptor as established. Keep line-profile transfer and final BSI/audio effect UNBOUND until the +0xc0f8 object kind and slot +0x28 target are typed.
