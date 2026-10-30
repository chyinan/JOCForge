# Local ProgrammeState v3 owner-availability contract

Status: USER_APPROVED_LOCAL_CONTRACT. Adopted 2026-10-30 Asia/Shanghai after direct user approval.

This v3 is a local JOCForge transaction-contract amendment. It does not change, overwrite, or reinterpret the approved v2 schema or its fixtures, and it makes no claim about DEE native behavior.

In METERING, the meter owner is INITIALIZED from its actual canonical state. The other eleven main owners are UNINITIALIZED and carry no generation, hash, or state reference. `measurement_receipt` is null.

A successful MEASUREMENT parent proposal must carry the actual terminal meter state, its matching typed measurement receipt, and eleven cold owner states produced by their approved constructors. The single parent swap preserves the meter state, installs the receipt and all eleven initialized owner references, and transitions to READY with one phase-generation increment. Each owner construction receipt records `before_availability=UNINITIALIZED` and the actual after-state identities.

If any identity, provider, terminal getter, or cold-owner construction fails, the parent installs none of the proposed after-state and does not enter READY. Fatal handling and full restart follow the approved GTC lifecycle. The accompanying transition cases are input-only and contain no production expected hashes.

Adopted source proposal manifest SHA-256: `2b938b927a918a46842acc4ab1b10017e98093c861dd4905739f36e294268a2d`.
Preserved approved v2 manifest SHA-256: `8a7410f0d5049d40196c3baf3f4a074d1693e11c65517440cb6f5368632a369e`.

The v3 schema and typed availability projection do not claim that a production Meter receipt, READY root swap, AU, or output file has run.
