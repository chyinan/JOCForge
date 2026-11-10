---
status: sanitized-candidate-contract
topic: "Flat7x F15 attempt, paired-AU staging and receipt boundary"
spec: "Wash-reviewed architecture decisions AD-003/005/008/010"
confidence: bounded architecture contract; production admission remains closed
tags: [JOCForge, Flat7x, F15, GTC, staging, receipt]
audience: implementer
---

# Flat7x F15 staging transaction contract

## Package scope

This package carries the approved attempt/fail-stop and Flat7x pair invariants needed to continue the target-only F15 model. It is bound to isolated commit `f82e8e452abcf822913d60976ba730fef01de721` and the source-document hashes in `SOURCE-RECEIPTS.json`.

The current candidate row is I0 640 kbps followed by D0 1024 kbps (6,656 aggregate bytes). This is the selected implementation target row, not a measured rate for the official demo file. It does not grant production output or resolve the official demo's exact rate.

## Source and index domains

- I0 and D0 in one AU must carry the same source-generation token. A pair with unequal source generations is rejected as a whole.
- `PublishedAuIndex` is the sequential index of the output AU. It remains distinct from the source-generation value and the local F15 owner generation. Do not equate these numbers or convert one token domain into another.
- The current byte-sequence and staging receipts accept caller-provided source-generation labels. They do not bind those labels to the source callback owner; that join remains `UNBOUND`.

## F15 staged state

For the target-only plan, a cold owner has a caller-supplied attempt identity, expected AU count, next `PublishedAuIndex=0`, local owner generation 0, durable length 0, and an empty pair ledger. Each proposal represents exactly one complete ordered I0+D0 pair, with separate I0/D0 lengths, digests and CRC residuals plus the aggregate AU digest and offset.

The proposal is a staged value. Its receipt candidate records one staging-file write/readback operation; it is not a GTC `OwnerReceipt`, a source callback receipt, or final-file exposure authority. The current Flat7x owner is additive and remains outside the selected 5.1 `OutputOwner`/GTC projection.

The generic owner receipt schema requires `owner_id`, `transaction_id`, before/after generation, before/after canonical identity, `changed`, and `commit_count`. A Flat7x GTC owner projection has not been approved; do not synthesize or register one from the candidate F15 types.

## Failure, EOF and exposure

- A missing I0 or D0 frame, frame-role receipt, metadata/integrity receipt, unequal pair source generation, or EMDF container above 511 bytes rejects the entire AU. I0-only publication is forbidden.
- F15 proposals may be appended only to a caller-owned staging file at the expected offset. The adapter checks pair extents and CRCs, writes the whole candidate AU, syncs the file and reads back the same range before returning a receipt candidate.
- A partial write, sync error, readback mismatch or receipt mismatch can leave bytes in staging. The caller abandons that attempt; the adapter does not roll back, resume, rename, finalize or expose a final destination.
- Per AD-005, fatal failure retires the whole attempt, leaves committed owner identity unchanged, and recovery starts from a fresh programme-origin root. No same-attempt retry is authorized by this contract.
- Per AD-010, the complete ordered AU is the publication unit. Final exposure stays behind full-attempt completion, required receipts, and a future Wash-approved GTC/finalize owner contract. The current IO adapter is only a staging/readback seam.

## Required open gates

`production_allowed=false` remains in force. Before production connection, Wash must separately bind:

1. Source callback/PCM generation to the same-generation I0+D0 frame pair and its `PublishedAuIndex` admission.
2. The Flat7x F15 canonical state/GTC owner projection and receipt/commit path.
3. Attempt EOF/finalize/final-exposure receipt semantics and the actual destination staging policy.
4. I0 exact BSID/BSI; D0 BSI, audio/role owner and native audio algorithm; complete all-AU EMDF fit in D0 block-2 single-skipfld.

These gates keep the existing 5.1 owner and publication path unchanged.
