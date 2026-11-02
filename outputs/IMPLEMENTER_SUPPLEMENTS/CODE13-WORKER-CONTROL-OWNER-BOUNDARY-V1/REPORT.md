# Code13/14 worker control-table owner boundary

## Newly bound native edge

On the Atmos DLL's normal setup path, `FUN_1801b0750` constructs a `{16,8}` control-table owner. Its `+0x60` table contains 16 slots of 16 bytes (`data pointer`, `status U32`, `type U32`); each slot reserves eight 30-byte subrows, initially zeroed. `FUN_1801c22d0` returns the owner pointer in both `outer+0x42f8` and `header+8`. `FUN_1801bfd90` writes that owner to `worker+0x182f0`; `FUN_1801c3490` later passes the same worker to `FUN_1801c0770`, which reads `worker+0x182f0`. This proves the field can be non-null in a successful DLL setup path and binds a same-worker consumer.

The decoded control-stream writer can create status/type rows (including type 3 and bit-index types through 16). Under the code14/code13 gate, the consumer scans these rows and may omit selected source pointers before per-object processing. This is an audio-selection-capable path, not merely a BSI write.

## Still unbound

No same-owner join binds the separate `FUN_1801ffe20` record-writer header to this worker owner; the control stream is not yet mapped to named ADM AudioObject/ChannelFormat/CHNA rows; the host StageRunner route-to-factory edge is unbound; and no run token ties the owner to a selected input generation. Therefore the static worker table is not yet an implementable ADM row filter for the current JOCForge path.

## Next minimum observation

Use one metadata/pointer-only run per authorized 38-row `人是猫` and 36-row My Jealousy sample. Capture StageRunner route/index and task token, worker/owner/header pointers, table slot status/type, control-stream owner, selected skip indices, and source reader generation. Join only on explicit same-run pointer/task identity; do not read subrow payloads or PCM. The source package checker passed normally and with `python -O`; no DEE run occurred in that slice.
