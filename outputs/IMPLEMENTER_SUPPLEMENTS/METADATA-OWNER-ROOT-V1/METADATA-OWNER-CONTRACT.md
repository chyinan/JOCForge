# Metadata owner root

## Scope and evidence class

This candidate covers the selected 48 kHz, 1536-sample, four-payload profile.
Timer, queue parameters, queue order, serialization order, writer lineage,
suppression, capacities and no-flush EOF are frozen-model facts. The root state
serialization and receipt wrapper are clean-room interface design. No native
DEE receipt object or global root serialization has been observed.

METADATA_SOURCE_RECEIPT=READY_CANDIDATE

METADATA_EOF_PROPOSAL=READY_CANDIDATE

MEASUREMENT_ROOT_RECEIPT=BLOCKED

FINALIZE_ROOT_RECEIPT=BLOCKED

RESTART_ROOT_RECEIPT=BLOCKED

The blocked global receipts must be specified by the programme coordinator.
They may not be replaced by the integrity receipt or inferred as native.

## Authority, inputs and clocks

F13 is the sole executor and commit owner for the ID1 timer, ID11/ID1/ID2/ID14
queues, fixed carriage mode, queue buffer IDs, writer/pop generations and EMDF
assembly proposal. Upstream supplies immutable ID11, ID2 and ID14 bodies tagged
with the current writer generation. ID1 is calculated from the current
programme measurement config and the staged timer state.

For source s, dispatch is s+1; AU is absent for s<2, otherwise s-2.
Every supplied writer generation equals s. Queue mutation order is
ID11, ID1, ID2, ID14, each push immediately followed by pop. Publication order
is ID11, ID14, ID2, ID1. A published source consumes ordinary writers s-2
and ID14 writer s-1.

## Complete deployed wrapper configuration

metadata-wrapper-config.json is a required provider asset. Both its raw file
SHA-256 and sorted compact JSON SHA-256 are pinned by
metadata-public-provider.json. Every call uses the
same programme-fixed object; wrapper fields are inputs and are never inferred
from expected EMDF bytes.

ID11 is sample_offset null, duration null, group_id 0,
codec_data_present false, discard_unknown_payload false,
payload_frame_aligned false, and create/remove/priority/processing null.

ID1 is sample_offset null, duration 1536, group_id null,
codec_data_present false, discard_unknown_payload false,
payload_frame_aligned false, and create/remove/priority/processing null.

ID2 is sample_offset null, duration null, group_id null,
codec_data_present false, discard_unknown_payload true; every field after
discard is unconsumed and therefore explicitly null.

ID14 is sample_offset null, duration null, group_id 0,
codec_data_present false, discard_unknown_payload false,
payload_frame_aligned true, create/remove false, priority 0 and
processing_allowed 0.

The ID1 timer output determines the ID1 payload body fields and bytes. The
outer wrapper duration is the independent fixed value 1536 samples. It is not
read from timer output, does not alter timer state and is not encoded inside
the timer payload fields. Their shared fact is only that one body and wrapper
are produced for each 1536-sample source call, including priming calls.

## Committed state and canonical bytes

Semantic field order is profile ID, provider SHA-256, generation, next source,
fixed carriage mode, timer state, four queues in numeric owner order
ID1/ID2/ID11/ID14, finalized, pending disposition.

Canonical bytes are defined by metadata_root.py state_bytes:

1. MetadataStateV1 plus zero byte;
2. U16LE length plus UTF-8 profile ID;
3. raw provider SHA-256;
4. generation and next source as U64LE;
5. U16LE length plus UTF-8 carriage mode;
6. six timer remaining S64LE, six cached S64LE, mode and counter S64LE;
7. four queues in ID1/ID2/ID11/ID14 order: ID U8, capacity U16LE, delay
   U32LE, frame U16LE, write slot U16LE, slot count U16LE; each slot is a
   presence U8 and, if present, writer U64LE, payload length U16LE and payload;
   then one U16LE buffer ID per slot;
8. finalized U8;
9. pending count U16LE, then each pending entry ordered by queue ID and slot:
   queue ID U8, writer U64LE, raw 32-byte payload SHA-256 and UTF-8 disposition
   prefixed U16LE.

State identity is SHA-256 over these bytes. Provider identity is SHA-256 over
sorted compact UTF-8 JSON.

### Timer receipt digest

The timer digest has its own explicit clean-room domain. Its preimage is ASCII
MetadataTimerStateV1 followed by one zero byte, then exactly fourteen signed
64-bit little-endian integers: remaining[0] through remaining[5], cached[0]
through cached[5], mode, counter. SHA-256 over that preimage is the timer
digest.

It contains only timer runtime state. It excludes period/provider identity,
root generation, source, dispatch, AU, callback, programme, attempt and
transaction identity. Those identities remain separately authenticated by the
metadata root state and receipt fields. The timer bytes after the prefix are
byte-for-byte identical to the timer block embedded in canonical root state,
but the digest is not the SHA-256 of the bare root timer block because the
timer-specific domain prefix is included.

This digest wrapper is clean-room interface design, not an observed DEE
serialization. Cold and source0 preimages are materialized in vectors and
independently recomputed by verify_timer_digest.py.

## EMDF terminal bit layout

All fields use MSB-first bit order in one continuous body bitstream. Payload
sections are not independently byte padded. After the last payload body, write
ID0 as U5, primary protection length code as U2, secondary protection length
code as U2, primary protection bytes, secondary protection bytes, then zero
bits only as needed to reach the next whole EMDF body byte.

Primary code mapping is code1=1 byte, code2=4 bytes, code3=16 bytes; code0 is
not admitted by the selected encoder. Secondary mapping is code0=0 bytes,
code1=1 byte, code2=4 bytes, code3=16 bytes. The selected public handoff writes
primary code2 followed by four zero placeholder bytes and secondary code1
followed by one zero placeholder byte. F14K later replaces those slots.

Bit positions are absolute from the first sync byte at bit zero. Every receipt
records terminal ID start, both code starts, primary/secondary protection
ranges, padding range and wire/body byte lengths. EMDF-TERMINAL-FIXTURES.json
contains an input-defined empty/cold terminal and the first published source2
example. verify_emdf_terminal.py reconstructs both without importing the root.

## Transaction and receipt

All bodies and writer generations pass the readiness barrier before the first
queue mutation. Work occurs on one cloned state. The timer advances and emits
ID1, then all four push/pop operations execute. Publication and EMDF assembly
are validated before the proposed after-state becomes eligible.

One metadata receipt contains before and proposed-after state identities,
source/dispatch/AU clocks, timer before/after identity, all four ordered queue
operations, popped writers, publication order, EMDF identity when published,
and commit count one. The enclosing source transaction publishes this receipt
and swaps the metadata root once. The leaf does not claim a separate durable
commit.

Any body, generation, timer, capacity, occupied-slot, lineage or assembly
failure discards the clone and receipt. Committed bytes and digest remain
unchanged, commit count is zero, and the programme attempt becomes RETIRED.
The same attempt cannot retry. A new attempt validates the provider and starts
from cold state.

## EOF

EOF validates the exact source count. It does not push, pop, advance timer,
assemble EMDF or create an AU. Pending entries are recorded with queue, writer,
payload hash and DROP_AT_DONE; queue storage is then released without
advancing queue clocks. Generation increments once and finalized becomes true.
Further source or EOF calls fail.

This metadata EOF after-state is a leaf proposal. The global FINALIZE receipt,
durable output transition and RESTART receipt remain blocked as stated above.

## Acceptance

The vectors materialize five continuous source calls, the four queue operation
order, timer and writer generations, source suppression, three published EMDF
proposals, EOF pending disposition, cold reset and retry. Negative vectors
cover each body capacity, each writer mismatch, readiness failure, timer
failure, occupied destination, source/dispatch/AU lineage and post-final call.
Fault plans are test-harness boundaries and are not production parameters.

Every negative vector carries its complete committed prestate, canonical bytes
and state SHA. In occupied_destination, queue ID11 slot0 contains writer 99 and
payload byte aa while every other cold field remains explicitly materialized.
The source0 call passes identity/body readiness, then fails on the first queue
operation before overwriting that slot. Expected after canonical bytes equal
before exactly; receipt is null, commit count is zero and the programme result
is RETIRED.
