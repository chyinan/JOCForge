# F15 final exposure terminal contract

## Result

Attempt 08 passed its logged EOF source/AU ledger but failed the F15 publication transaction at `ParentFsyncAfterRename` with application exit code 2. It emitted no `FinalExposureReceipt` and never reached `DONE`; that output is not eligible for the requested exact-hash acceptance check.

## Contract-derived success conditions

For the selected plan, the terminal must remain bound to the same programme, input, profile, provider, attempt, staging owner, and plan-derived extents. Every AU append must be contiguous, 4096 bytes at `au_index × 4096`, read back equal to its proposal with CRC residual 0, and flushed/fsynced before a `DurableAppendReceipt` and parent ledger commit.

Finalization requires a validated EOF ledger and pending state, staging data fsync, a no-overwrite atomic namespace operation, and parent-directory fsync. Only then may a `FinalExposureReceipt` bind the attempt, staging owner/generation, product identity/length, and durable-ledger identity. The parent may then commit final identity and `DONE`.

The source package did not contain the referenced `OUTPUT-OWNER-ROOT-V1` schema payload. Exact serialized field names remain `UNBOUND`; this report records only the contract-level meanings. The same-named unapproved test fixtures were not inspected. Repair commit `ab8770f` was not visible in hash-bound receipts/contracts, so its effects remain uncaptured.

## Scope

This is an output-publication transaction contract, not an audio-algorithm rule. It does not claim that the current F15 implementation or repair commit has passed. It defines when a completed file is eligible for the exact comparator hash check; it does not authorize opening or decoding the file contents.
