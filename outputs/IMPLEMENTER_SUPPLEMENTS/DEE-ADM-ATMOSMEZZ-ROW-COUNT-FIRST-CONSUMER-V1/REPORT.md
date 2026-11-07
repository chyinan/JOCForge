# DEE AtmosMezz native row-count consumer boundary

## Bound static path

The inspected source descriptor `K` count is computed from individual-record count plus group-entry counts, accumulated as U32 and stored as low U16. A conditional composite descriptor `B` is then formed: one branch copies six-byte rows and its count from a source adapter; another synthesizes rows from tag-255 counts. The selected branch and actual count were not captured.

On the descriptor-bearing reader path, the U16 count at `B+0x0A` is zero-extended into `open_config[2]` as U32 `N`. The first explicit numeric gate is `1..128` inclusive. `N` initializes N row-map entries over a source-map backing capacity of 128; physical descriptor-copy span is `6*N` bytes. The later per-frame source-map loop still iterates 128 source slots and uses populated entries/flags; it is not a 36-, 38-, or 48-row loop.

## Limits

This establishes a conditional dynamic count path and a static `1..128` guard for the inspected descriptor-bearing Atmos DDP branch. It does not prove arbitrary ADM admission or every count in that range. `WAVE fmt.channels`/CHNA row count to the concrete source `K` instance and generation remains `UNBOUND`; runtime counts, physical row-storage reservation and PCM sample-row allocation were not evaluated. The separate canonical count `M=effective_object_count+10` is a different domain; equality with N in 36/38 metadata examples must not be generalized.

No DEE execution, parser replay, dynamic capture, PCM/payload/DBMD/key/license read, or implementation access occurred.

## Provenance

Source package `ADM-ATMOSMEZZ-ROW-COUNT-FIRST-CONSUMER-V1` manifest SHA-256 is `d9e45e2519718ed154fcc5c5ec9e1692434ccdeec64585a47a38ef918328d63a`; its checker passed normally and under `python -O`.
