# DEE WAVE/CHNA to reader-owned U group-row lineage

## Bound input and row path

The factory-created ADM/BWF reader state `U` stores the WAVE fmt channel count and the parsed CHNA reader/count. In the inspected `U.open` path, fmt count is compared with the CHNA entry count; mismatch enters an error path. This is input-format validation, not a direct assignment to the later K count.

For an ADM bed/object channel record, the CHNA UID lookup returns a one-based WAVE physical track index; the caller subtracts one and stores the zero-based physical slot in the U source map and in the typed `U Bed::Channel` record at record `+0x10`. The record is appended to its bed group vector, then the completed group is appended to U's group-record container. A separate previously sealed K-count receipt consumes U group-entry counts.

## First break and limits

This closes a static indirect lineage from CHNA-resolved physical slots through U group entries to the input domain of K's count producer. It does not bind a concrete file's count, actual U/K owner-generation, whether every CHNA entry becomes exactly one K row, the six-byte K group-row projection, or all input/open branches. The static 1..128 gate remains a conditional code limit, not a runtime arbitrary-track support claim.

No DEE execution, WAVE/CHNA/PCM payload read, DBMD/key/license read, parser replay, or implementation access occurred.

## Provenance

Source package `ADM-WAVE-CHNA-U-ROW-CONTAINER-V1` manifest SHA-256 `7af3ea125b1029bf9407a10c20fee4f6dd70807ec93b2e92d5bc2865141474aa`; normal and `python -O` checker passed.
