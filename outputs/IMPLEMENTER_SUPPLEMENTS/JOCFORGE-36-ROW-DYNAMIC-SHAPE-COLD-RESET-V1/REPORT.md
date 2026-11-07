# JOCForge dynamic ADM row shape and cold reset contract

## Implementable 36/38 row model

For the selected 5.1 profile with ten bed rows and one direct LFE, keep these count domains separate:

- Physical ADM input rows: N, taken from the current prepared graph.
- Bed rows: 10 for this selected profile.
- Object rows: O = N - 10.
- Regular non-LFE rows: R = N - 1.
- Active global source IDs: S = 11 + R = N + 10.

Concrete vectors:

| N | bed | object | regular | active global sources | global IDs |
|---:|---:|---:|---:|---:|---|
| 36 | 10 | 26 | 35 | 46 | 0..45; 46/47 absent |
| 38 | 10 | 28 | 37 | 48 | 0..47 |

At N=36, do not manufacture channels or set IDs 46/47 to zero. Represent the missing IDs as absent with an explicit presence/Option state, or use a compact 46-source vector plus an explicit source-ID map. At N=38 the current 48-wide axis is full. N=39 requires a 49-source axis; the current fixed 48-wide shape is an internal product ceiling, not proof of a DEE/native ADM limit.

Physical row identity must pass through an explicit typed mapping to codec/global source identity. Do not infer it from row ordinal, equal counts, or matching hashes. The channel-identity edge remains a separate dependency being investigated.

## Cold/reset and callback state

For a new job/attempt, discard prior sampler, binding, snapshot, callback counter, and row-history state and construct fresh owners from the selected PreparedJobInput graph. Do not resize an existing 38-row owner in place to 36. Within one attempt, keep each owner's row identity and generation stable across callbacks; preserve recurrence/history between callback blocks and reset only at the new-attempt boundary.

Apply the selected graph's N/O/R dimensions consistently to Meter normal buffers, FreshZero buffers, F01E normal-control rows, SpatialControl rows, and the regular-history extent. For 48 kHz with 1536 samples per callback, regular history has R * 1536 F32 samples; at N=36 that is 35 * 1536, and at N=38 it is 37 * 1536. Keep Meter and Main Spatial owner receipts distinct while binding each to the same prepared job/attempt and row-identity map.

The global source-axis size is a separate domain from physical rows. A 48-slot sparse axis is valid for N=36 only when absent IDs 46/47 remain explicit and all consumers honor presence. Dense ReaderTarget and 38-row cold-root guards must be widened or replaced along the selected production path; changing the public parser alone is insufficient.

## Current limits and status

The hash-bound source audits show the first selected production gate is Meter G0's fixed 38/114-byte source map, followed by a legacy F01E 38/28/37 normal snapshot and FreshZero 38-row guard. The dynamic continuation proposal exists locally but was not connected to the selected callback/SpatialControl path in the audited source snapshot.

The 48-wide source axis is the current internal limit. The audited U8 active-source count overflows at N=246 (S=256), and the highest U8 global ID overflows at N=247. Full arbitrary-N support therefore needs dynamic axis storage and wider count/ID fields; the 36-row MVP contract does not claim every N is already supported.

No 36-row CLI encode, DEE execution, PCM comparison, or playable output is established by these source/model packages. This contract fixes the model row shape for implementation and preserves the known absent-ID rule; native source-index mapping and PCM parity remain independent checks.
