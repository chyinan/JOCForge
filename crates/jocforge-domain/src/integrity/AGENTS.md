# Integrity processing

Last verified: 2026-10-20

## Purpose

Keep the selected public keyless integrity profile in the pure domain layer. The local owner tracks source order and completion without holding a checksum accumulator.

## Contracts

- Accept only `keyless-crc32-crc8-crc16.v1`; every other profile returns `UNSUPPORTED_PROFILE`.
- Validate the canonical public provider before cold admission and before each source call.
- Run SOURCE, DISPATCH, and AU provisionally, then commit local owner state once after AU succeeds.
- On failure, expose no frame or stage receipt, preserve committed state, and retire the attempt.
- EOF requires the exact expected source/AU counts. It finalizes state without emitting a frame or receipt.
- Local integrity commit does not implement global CallbackTransaction commit or durable output publication.

## Dependencies

- Uses `timeline` clock newtypes, sibling checksum modules, `sha2`, and `serde_json`.
- Receives provider and frame bytes from callers. This module performs no file, environment, process, or network I/O.

## Key decisions

- Public CRCs detect corruption; they do not provide authentication.
- Private HMAC/vendor authentication and native receipt wire formats remain blocked.
- Test-only stage faults are compiled only for tests and are absent from the production API.

## Invariants

- SOURCE uses CRC32/ISO-HDLC for the primary slot and CRC8/SMBUS for the secondary slot.
- DISPATCH inserts the patched container only into a zero carrier inside `frame[2:4094]`.
- AU writes E-AC-3 CRC16 big-endian at `frame[4094:4096]` and requires residual zero.
- Source indices 0 and 1 run all integrity stages but do not publish AUs.
- No failure path changes canonical committed state or publishes provisional receipts.

## Key files

- `crc16.rs` - E-AC-3 CRC16 generation and residual verification.
- `keyless_checksum.rs` - selected provider validation, CRC32, and CRC8.
- `programme_owner.rs` - owner state, typed receipts, stage ordering, reset, and EOF.
- `programme_owner/tests.rs` - washed-vector replay and test-only failure injection.
- `../../../../docs/implementation/IMPLEMENTATION-MAP.json` - authoritative slice scope and validation evidence.
