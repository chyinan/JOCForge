# GTC owner state-ref convention

Status: `APPROVED_LOCAL_SNAPSHOT_REFERENCE_CONVENTION`.

The selected global transaction contract requires a non-empty `state_ref` in each `ownerState`, but its JSON schema does not define the string syntax. This supplement fixes that local representation convention for every registered owner without changing any owner state bytes or algorithm.

## Rule

For the registered `owner_id` value `X`, set `state_ref` to the exact UTF-8 string `owners.X`. The period is a literal separator. This is an opaque logical member name inside the complete immutable programme-state snapshot. It is not a filesystem path, URI, JSON Pointer, digest, or owner state serialization.

The referenced member contains that owner's complete typed state. `state_sha256` and `cold_sha256` continue to identify canonical state bytes as defined by the owner's state contract. The reference does not replace those bytes or hashes, and it does not determine generation, callback order, transaction identity, or commit authority.

This applies to every owner id in the approved GTC schema: `reader_eof`, `frontend_control`, `spatial_control`, `spatial_audio`, `joc`, `meter`, `drc`, `analysis`, `codec`, `metadata`, `integrity`, and `output`.

Examples:

- `owner_id=codec` → `state_ref=owners.codec`
- `owner_id=meter` → `state_ref=owners.meter`
- `owner_id=reader_eof` → `state_ref=owners.reader_eof`

Derive the reference from the validated registered owner id; do not accept a separately supplied path string that could disagree with `owner_id`.

## Scope boundary

This is a local GTC representation choice because the GTC is a clean implementation transaction contract. It is not a claim about DEE-native state-ref syntax. It does not define or authorize any owner's state fields, serialization, hash timing, or production readiness.
