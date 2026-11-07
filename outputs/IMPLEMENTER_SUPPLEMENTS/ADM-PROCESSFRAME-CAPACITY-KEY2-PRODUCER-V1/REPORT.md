# ProcessFrame capacity key-2 producer and arena boundary

## Key-2 producer

The hash-bound static path calls `FUN_140321500` for key `2`, producing ProcessFrame capacity field `C = root+0xC140`. A custom config is an I32 count/pointer pair at `config+0x3C/+0x40`, with `(I32 key, I32 value)` records. A matching key-2 record writes signed I32 C. Otherwise, the selected profile table supplies a signed I32 default.

In the bounded profile table, indexes 0, 1, 3, and 4 have identical key-2 rows: default 11, inclusive validation range 1..11, error `0xBA`. Indexes 2 and 5..13 have no key-2 row in the inspected table. The selected profile is `UNBOUND`; these table rows are not claims about the active job.

## Same-root and separate arena fields

`FUN_140314F50` takes a caller-owned root pointer and clears `param_3[0]` bytes before constructing the profile block at `root+0xC100`; key-2 output slot `+0x40` is the same root's `+0xC140`. The clear length is caller-supplied and has no minimum-size guard proving it covers C. If the selected profile lacks a key-2 row, the resolver returns 0 without writing its output slot and the caller ignores that status; active C therefore remains `UNBOUND` without a root-extent/profile join.

The root's `+0xC144` derives an internal slot count: C==2→3, C<7 and C!=2→6, C>=7→11. Per-slot request width is profile-dependent (indexes 12/13 use `0x400`, others `0x1800`) and request bytes are width×slot_count plus `0x1F`. The allocator uses a separate arena at `root+0xC188` (base) and `root+0xC180` (byte extent); its usable block capacity is not equated with C, M, or ADM N.

## Scope and risk boundary

The earlier M gate compares `M` to C, but the selected profile and actual C are not captured. If a profile row limited C to 1..11 and M were 36/38, the path would reject; neither premise is bound to an actual run, so this is a conditional risk, not a proven 36/38 failure. `N`, `M`, `C`, derived slot count, and arena bytes remain separate domains. No DEE run or media/PCM/payload/key/license access occurred.

## Provenance

Upstream manifest SHA-256 `502c4dfbe4010f70a4c7485f81d1c48fb780d73b8c8274b0dbdd86674cf74ff0`; package-seal SHA-256 `6a812cc6c3a12cbb4b4e8cedbbf5542c5f4d1ef0cfdad91b9b2807f37e211ba5`. Upstream normal and `python -O` checks passed. This supplement excludes Ghidra source/logs and binary/media payload.
