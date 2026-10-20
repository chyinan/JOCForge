# Public integrity programme owner root

## Readiness and boundary

`KEYLESS_PUBLIC_PROFILE=READY_CANDIDATE`

`PRIVATE_AUTH_PROFILE=BLOCKED`

This package closes the programme owner contract only for the public keyless
CRC profile `keyless-crc32-crc8-crc16.v1`. It contains no private key, vendor
HMAC rule, inferred substitute, or authentication history. A caller requesting
private authentication must receive an unsupported-profile error.

The public CRC algorithms and their byte coverage are frozen-model facts. The
programme state encoding and receipt wrapper are clean-room interface design,
not a recovered DEE wire format. No native receipt object was observed.

## Inputs and outputs

One call consumes exactly one identity tuple and these bytes:

1. `identity = (programme_id, attempt, transaction_id, source_index,
   dispatch, au_index)`;
2. one zero-slot integrity container and the primary/secondary slot offsets;
3. one 4096-byte frame template and the container offset;
4. the canonical public provider and its SHA-256 identity.

The identity relation is exact: `dispatch = source_index + 1`; `au_index` is
null for source indices 0 and 1, then `source_index - 2`. The first two calls
still execute SOURCE, DISPATCH, and AU integrity work, but their AU receipts say
`published=false`. From source index 2 onward, `published=true`.

A successful call returns three ordered receipts, a completed 4096-byte frame,
one proposed after-state, and exactly one commit. A failed call returns no
receipt, no frame, no state change, and zero commits.

## Public checksum pipeline

SOURCE requires zeroed four-byte primary and one-byte secondary slots. It
computes CRC32/ISO-HDLC over `zero_slot_container[4:]`, writes the big-endian
four-byte result into the primary slot, computes CRC8/SMBUS over those four
bytes, and writes that byte into the secondary slot.

DISPATCH requires the selected carrier range in the 4096-byte template to be
zero and fully inside `frame[2:4094]`. It inserts the patched container without
changing any other byte.

AU computes CRC16 with polynomial `0x8005`, init zero, non-reflected, xorout
zero over `frame[2:4094]`, writes it big-endian at `frame[4094:4096]`, and
requires the CRC over `frame[2:4096]` to have residual zero.

## Committed state

The committed state fields, in semantic order, are:

1. `profile_id`;
2. `provider_sha256`;
3. `generation`;
4. `next_source_index`;
5. `published_au_count`;
6. `finalized`.

Cold state has generation, next source index, and published count all zero,
with `finalized=false`. The provider is validated before cold state is
admitted. There is no cross-frame checksum accumulator.

Canonical state bytes are exactly:

1. ASCII `IntegrityStateV1` followed by one zero byte;
2. profile byte length as unsigned little-endian 16-bit;
3. UTF-8 profile bytes;
4. raw 32-byte provider SHA-256;
5. generation, next source index, and published AU count as three unsigned
   little-endian 64-bit integers;
6. finalized as one byte, zero or one;
7. one reserved zero byte.

The state digest is SHA-256 over those bytes. No JSON serialization participates
in the state digest. The provider digest is SHA-256 over UTF-8 JSON with keys
sorted, no whitespace, and separators `,` and `:`.

## Receipts and atomicity

The receipt schema is `INTEGRITY-STAGE-RECEIPT.schema.json`. Each stage receipt
names both the committed before-state and the same proposed after-state. The
three receipts remain provisional until AU succeeds. Only then may the root
publish all receipts and replace the committed state once.

On any identity, provider, shape, slot, carrier, CRC, or injected stage failure,
the root discards every provisional byte and receipt. It returns the exact
before-state digest, marks the programme attempt `RETIRED`, and forbids retry in
that attempt. A retry starts with a new attempt identity and cold state.

## EOF and reset

EOF is accepted only after the exact expected number of source calls and
published AUs. EOF emits no integrity frame, receipt, or metadata-only flush.
It increments generation once and sets `finalized=true`. Further calls and a
second finalization fail.

Reset discards the old attempt and recreates cold state from a freshly validated
provider. State from a retired or finalized attempt is never reused.

## Acceptance evidence

`INTEGRITY-ROOT-VECTORS.json` contains five continuous calls, including two
suppressed source clocks and three published AU clocks, plus ten negative cases,
reset, finalization, and retry behavior. `verify_integrity_vectors.py` is an
independent calculation and does not seed itself from expected output bytes.

Implementation readiness is limited to the keyless public profile and requires
the washed package to preserve every field and boundary above. Private
authentication remains `BLOCKED`, and no implementation may guess it.
