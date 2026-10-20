# Output owner root

## Status and evidence boundary

OUTPUT_PROPOSAL_ROOT=READY_CANDIDATE_TEST_ONLY

LOCAL_SYNTHETIC_FILESYSTEM_SHELL=PASS

PRODUCTION_ALLOWED=NO

REAL_4249_SOURCE_PUBLICATION=NOT_RUN

POWER_LOSS_RECOVERY=NOT_RUN

PRODUCT_PATH_EXECUTION=NOT_RUN

PRIVATE_AUTH_HMAC_VENDOR_KEY=EXCLUDED

NATIVE_RECEIPT_SCHEMA=NOT_OBSERVED

This root owns the attempt-scoped staging stream, durable AU ledger and final
exposure proposal. It does not own codec, metadata, integrity, source audio or
global CallbackTransaction state. Metadata EOF may supply expected counts but
cannot serve as a durable output receipt.

## Clocks and authority

For source s, dispatch is s+1 and AU is null for s<2, otherwise s-2.
Suppressed sources advance the output source clock but perform no append.
Published AU indices are consecutive and offset is au_index*4096.

The input frame must be exactly 4096 bytes, match its declared SHA-256 and have
CRC16 residual zero over frame[2:4096]. These checks complete before physical
append. The root never repairs codec or integrity bytes.

## Committed state and canonical serialization

Semantic fields are profile ID, provider SHA-256, attempt ID, staging name,
generation, next source, published count, durable length, ordered durable AU
ledger, finalized flag, product name and final product SHA-256.

Canonical bytes are:

1. ASCII OutputStateV1 plus NUL;
2. profile ID, attempt ID and staging name as U16LE length plus UTF-8;
3. raw provider SHA-256;
4. generation, next source, published count and durable length as U64LE;
5. ledger count U32LE, then each entry: AU U64LE, offset U64LE, length U32LE,
   raw frame SHA-256 and CRC residual U16LE;
6. finalized U8;
7. product name as U16LE length plus UTF-8;
8. final SHA presence U8 and raw SHA-256 when present.

State identity is SHA-256 over these bytes. Cold state validates the provider,
uses generation/source/count/length zero, an empty ledger and finalized=false.

### Durable ledger digest

ledger_sha256 is not a JSON hash. Its normative preimage is ASCII
OutputDurableLedgerV1 followed by one NUL byte, U32LE entry count, then entries
in AU order. Each entry is au_index U64LE, offset U64LE, length U32LE, raw
32-byte frame SHA-256 and crc_residual U16LE. SHA-256 over exactly those bytes
is ledger_sha256. No path, attempt, generation, source or provider field is
included; those remain in the output root state and finalize proposal.

The synthetic vector supplies the complete preimage hex, byte count and SHA.
The independent verifier reconstructs it from the input ledger.

## Pure proposal and typed receipts

prepare_source validates identity and frame. Suppressed calls propose only the
next source/generation. Published calls return AppendProposal with attempt,
source/dispatch/AU, expected offset, 4096-byte frame identity, before state and
proposed ledger entry. It does not mutate state or perform I/O.

The imperative shell consumes exactly that proposal. DurableAppendReceipt is
valid only after append loop completion, exact length, readback byte/hash/CRC
verification and data fsync. The root then commits the proposed state once.
Short positive writes are continued until 4096 bytes; a zero/negative write is
fatal. A failed shell receipt cannot advance the ledger.

## Physical ordering

For each published AU:

1. verify committed staging length equals proposal offset;
2. seek to offset and loop writes until all 4096 bytes are accepted;
3. verify resulting length equals offset+4096;
4. read back exactly 4096 bytes;
5. verify byte equality, SHA-256 and CRC residual;
6. flush user buffers and fsync the staging file;
7. emit DurableAppendReceipt;
8. commit the in-memory output root once.

Finalization proposal requires exact source/AU/count/length ledger and no
pending append. The shell orders:

1. re-open and verify staging length and complete product SHA-256;
2. fsync staging;
3. write a finalize-intent containing attempt, state SHA, product length and
   product SHA; fsync the intent;
4. atomically replace staging to the requested product path;
5. fsync the parent directory;
6. emit FinalExposureReceipt;
7. commit finalized=true and final product identity.

The local probe only tests this order with ordinary files. Filesystem-specific
atomicity, directory fsync support and power-loss behavior are not qualified.
If replace or parent fsync fails, the attempt is RETIRED and requires external
recovery inspection. This package does not claim automatic roll-forward or
rollback.

### Finalize-intent canonical bytes

The intent object has exactly four fields: attempt_id string, product_length
non-negative integer, product_sha256 lowercase 64-hex string and state_sha256
lowercase 64-hex string. Its file bytes are UTF-8 JSON with keys sorted,
separators comma and colon, ensure_ascii=false, no BOM and no trailing newline.
The test vector includes the input object, complete canonical UTF-8 hex, byte
count and SHA-256. LOCAL-FILESYSTEM-PROBE file size/hash are observation
records; acceptance comes from reconstructing bytes from this normative input
and comparing them to the observed file.

## Failure, reset and EOF

Pre-append validation failure leaves state and files unchanged. Failure after
physical bytes were written abandons the attempt staging artifact; committed
state remains unchanged and same-attempt continuation is forbidden. A new
attempt receives a new staging identity and cold ledger.

EOF emits no frame. It checks exact configured counts and prepares
finalization. There is no metadata drain or additional codec call.

Recovery vectors distinguish clean restart before any append, abandoned
partial staging, fully durable staging without state commit, failed replace
and uncertain parent-fsync outcome. They are test classifications, not proof
of production crash recovery.

### Readback-corruption test mutation

The test harness writes the proposed 4096-byte AU to
readback_corruption.partial, then patches the byte at absolute staging offset
proposal.expected_offset+17. The operation is XOR 0x01. The vector records
file name, absolute and frame-relative offsets, original byte, modified byte,
resulting length and resulting file SHA-256. The readback comparison must then
fail with READBACK_HASH before any DurableAppendReceipt or output state commit.
The independent verifier applies the same patch to the proposal frame and
compares the complete reconstructed bytes with the physical work file.
