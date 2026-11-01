# Attempt 07/08 F05/F13 transaction review

## Run outcomes

Attempt 07 stopped before EOF at `CallbackIndexMismatch` in the cached F01E/F03W/F04 parent proposal. The log and run receipt do not contain the failing callback/source ordinals, parent/source tokens, owner generations, or expected/actual operands. These remain `NOT_CAPTURED`.

Attempt 08 reached the aggregate source/AU EOF ledger: 4,249 sources, 4,247 published AUs, and 17,395,712 durable bytes. Its last logged append at callback 4250/source 4248/AU 4246 was durable. It then failed in F15 after rename at `ParentFsyncAfterRename`, exited 2, and emitted neither final exposure receipt nor `DONE`. Classify the source/AU evidence as `PASS` within the logged scope and terminal output as `F15_POST_FAILURE`, not success.

## Transaction model boundary

Current source carries delivery callback identity and payload/source-origin identity as separate fields, and source-control/geometry consumers check them against distinct clocks. The fields share the same nominal `CallbackLineageIdentity` type. Attempt 08 does not attest which exact source tree was compiled into its executable, nor does it capture per-source F05/F13 owner receipts. Thus the semantic split is visible in current source, while its causal role in attempt 08 is `NOT_CAPTURED`. The proposed new type-level model remains `MODEL_PROPOSAL_NOT_APPROVED`.

## Use

Reference-only evidence for distinguishing the pre-EOF attempt-07 callback mismatch from the post-EOF attempt-08 F15 publication failure. It does not authorize a new type contract, a code change, or an audio algorithm rule. No product artifact was opened, no encode was run/restarted, and no native disassembly was used. One search also matched a derived OpenJOC JSON; its result was discarded and is excluded.
