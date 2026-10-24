# ID11 native queue capacity and approved API limit

This supplement resolves the meaning of the selected native ID11 constructor capacity while preserving the separately approved 160-byte interface policy.

## Native queue field

For the selected constructor inputs, ID11 has `object_capacity=16`, `frame_capacity=8`, and a native per-descriptor payload byte limit of 2103. Its derived formula is `floor((123*O*F + 20*F + O + 903)/8) + 1`; with O=16 and F=8 this is 2103. The generic queue compares the descriptor's declared payload byte length against this U32 capacity before copying. 2104 returns queue status 5 and leaves the slot unoccupied. This is a per-descriptor payload limit, not an aggregate ring allocation or parser-state allocation. The independent parser-state allocation is 4763 bytes for these inputs; the three-slot payload storage is at least 6309 bytes plus descriptors/tables/alignment.

The 18 input-first cases cover raw queue admission at 69, 159, 160, 161, 2102, 2103 and 2104 bytes plus actual ID11 ingress cases with the same lengths. Native raw queue admission accepts through 2103 and rejects 2104. In the ID11 ingress 2104 case, parsing consumes its 552 semantic/alignment bits and returns success, queue insertion returns 5 and is ignored by the wrapper, and ingress returns success with the owner-valid flag set while the queue remains empty. The input is the observed 69-byte seed with zero bytes appended; it is not a naturally generated 2104-byte ID11 body.

## Approved 160-byte interface policy

Approved F13.4 declares 160 bytes as the selected interface/payload policy. R21 proves this is narrower than the native queue's physical per-descriptor storage limit; it does not prove that a natural selected ID11 producer emits more than 160 bytes. Keep these as separate fields: enforce 160 at the approved public API boundary, and model the native queue constructor/admission limit as 2103. Do not label 2103 as an aggregate memory extent or silently replace the public 160-byte policy.

If a future washed producer vector demonstrates a natural selected ID11 body above 160, reopen the public policy with that producer-bound evidence. Until then the approved 160-byte public rejection may remain; its rejection is local validation, not the native queue's 2104 status.

## ID6 and ID2 are separate

The generic ID6 group uses a three-byte per-descriptor queue limit; 0/2/3-byte pushes succeed and 4 bytes maps native queue status 5 to group status 3. This does not establish the semantic payload name for ID6. ID6 is not the separate ID2 owner, whose selected capacity is eight bytes.

The checker verifies queue admission/control observations in normal and optimized modes. It does not reimplement the ID11 semantic parser; seven ingress parser receipts are native consistency observations. No private key, authentication value, PCM, binary or decompiled source is included. Whole F13C and end-to-end readiness remain NOT_READY.
