# Metadata-control store distinct from F12 count producer

Status: `METADATA_CONTROL_TRANSFORMS_STATIC_CLOSED; F12_COUNT_OWNER_JOIN_UNBOUND`.

The reviewed encoder image contains a finite metadata-processing path derived from the worker descriptor/open configuration. It constructs a separate metadata-control state and optional metadata store; it does not establish that this state is the validated F12 configuration-count producer or the F12 working-header byte-119 owner.

## Static transforms

- A U16 channel count is zero-extended into an open-configuration U32 field. On the branch whose mode is not 1, `channel_count` six-byte mapping records are copied to scratch storage. Mode 1 omits that mapping and maps channel counts 10, 12, and 16 to codes 2, 1, and 0. The selected mode is not observed.
- A separate metadata-control state is constructed from two U32 configuration values with bounded constructor checks (first value 1..159, second 1..8). Its backing state is aligned and zero-initialized before two additional initializers; those initializer bodies are outside this bounded review.
- A metadata enable flag controls whether a metadata-storage reference is returned. When auxiliary metadata is non-null, mode 0 enables the path and maps auxiliary byte values 1, 2, 3, 4 to internal codes 0, 1, 2, 3. No actual auxiliary value or metadata body was read.
- When the canonical nine-row vector is present, the processor dispatches each 28-byte row, skips its first four bytes, and copies the remaining 24 bytes into a five-byte-per-slot configuration area. Three F32 inputs are converted to U8 through a bounded comparison/nearest-table helper; two other F32 inputs use clamped/scaled rounded-byte paths. The slot table's value meanings and numeric table entries are not decoded here.

These are metadata-control transforms, not an additional-BSI complexity formula. Do not assign semantic object-count or F12 complexity meaning to the skipped row prefix, channel count, auxiliary-mode code, or five-byte slot contents.

## Missing identity

No direct typed copy, remap, or passed-owner argument was found from this descriptor-derived metadata/control state to the validated F12 count producer input/configuration/record in the same source or reset generation. The last F12-related washed endpoint remains separate: a validated configuration-count path to working-header byte 119 and an 8-bit wire field. Keep the metadata-control path and the F12 count field separate until an explicit owner/call transfer joins them.

Selected mode and values, F12 writer entry identity, and any runtime generation join remain `UNBOUND`. This finite static slice does not establish a selected runtime, whole F13C readiness, or product acceptance.
