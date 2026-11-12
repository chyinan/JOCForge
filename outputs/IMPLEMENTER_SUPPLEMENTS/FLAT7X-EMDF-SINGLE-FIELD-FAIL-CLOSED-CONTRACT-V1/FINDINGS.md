---
status: washed-partial
topic: "Flat7x EMDF single-field capacity and fail-closed staging"
spec: "Approved Flat7x target contract and Wash-only native writer boundary"
confidence: high for target rule; native join unbound
tags: [JOCForge, Flat7x, EMDF, skipfld, fail-closed]
audience: implementer
---

# Flat7x EMDF single-field contract

## Implemented target rule

Place one complete EMDF container in one D0 block-2 `skipfld` U9 field. The container length must be at most 511 bytes. Validate the complete container and its length before adding any bytes to the staged I0+D0 pair. If the container exceeds the limit or cannot be represented in the single field, reject the pair before commit; do not fragment the container across fields, blocks, or access units.

## Native evidence boundary

Wash reviewed a DEE 6.7 candidate serializer and its generic bit sink. The observed serializer has a candidate bit-budget calculation and capacity/error paths; a separate output candidate writes a 16-bit count and uses staging caps. No typed join connects those values to the D0 block-2 U9 `skipfld`, complete EMDF bytes, or a same-generation AU. Native field capacity and native overflow behavior remain `UNBOUND`.

Therefore the user-approved 511-byte limit and `REJECT` behavior are the implementation contract. The candidate serializer's count, budget, staging caps, and status values do not define the target skip-field capacity.

## Pair staging behavior

Keep the existing 5.1 path unchanged. For Flat7x, construct the validated EMDF field as part of the D0 block-2 candidate, stage I0 then D0 under one pair receipt, and expose the pair only after all field checks pass. A failure leaves the candidate uncommitted.
