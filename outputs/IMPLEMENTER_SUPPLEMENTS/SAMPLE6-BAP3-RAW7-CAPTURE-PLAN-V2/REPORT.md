# Sample 6 BAP3 raw-7 capture plan — static only

Recipient: implementation evidence supplement  
Disposition: plan and instrumentation boundaries only; no raw-7 hit result exists yet.

## Verified source package

Wash independently recomputed the revised R5 package manifest, `MANIFEST.sha256`, `SHA256SUMS`, seal, and all three payload hashes/sizes. The revised manifest is `6d543ebdd3cd7f075c32c16232bb3d34901b5f2a5fbf5885faaf645b72a8b077`; seal file SHA-256 is `80f2d23d9ab644983253bb44672305d6627c32bbb313011a7d4e1d260744d04f`.

The plan expands raw-7 coverage to all three Ghidra-confirmed `FUN_140326700 -> FUN_1403264d0` call routes: `0x32680f`, `0x326872`, and `0x326929`. The special-tail route is kept distinct and has no quantizer-row assignment. The capture stop is `FUN_1403264d0+0x136` at RVA `0x326606`, after `CVTTSS2SI` and before saturation. The hit predicate is `ECX == 3 && R10D == 7`; native BAP3 saturation maps raw 7 to code 6.

The target gate is zero-based AU `[2750,2875)`. The plan associates quantizer events with the AU ordinal only through successful frame commits and requires runtime confirmation that the selected stream starts with block count zero and that successful commits map one-to-one to output AUs. If either check fails, capture must abort and remain unobserved. Raw pointers may be transient in-process join keys only; persistent output is limited to scalars and per-AU counts in the target interval.

## Execution status and limitations

No debugger has been attached and no DEE run has started. Therefore the sample's BAP3 raw-7 hit count is still `UNOBSERVED`, not zero. This package provides no input-only vector or numeric output; it is a static capture plan. It includes no PCM, encoded payload, private key/license material, Ghidra source, or decompilation text.

The plan is not a codec algorithm change and does not explain the measured LFE PCM mismatch. Its output, when captured, will be scalar evidence for comparison against the Rust model.

## Upstream seal

- Package: `bap3_raw7_capture_plan_wash_v1` (revised all-callsite plan)
- MANIFEST SHA-256: `6d543ebdd3cd7f075c32c16232bb3d34901b5f2a5fbf5885faaf645b72a8b077`
- package_seal.json SHA-256: `80f2d23d9ab644983253bb44672305d6627c32bbb313011a7d4e1d260744d04f`
- SHA256SUMS SHA-256: `bbdfb629c040fbe348f241a090d96ce4f8fddadac1dc54904d7b276a4b8194e4`
- Three payload files independently matched their manifest sizes and hashes.
- This revision supersedes the initial static plan manifest `e121d91fa4cdbe5020276632dfd4ffa655b8ca96d9f9d4ad511e68a55345de04` / seal `b6857d98cbd2f214bd53e68dba629c56d509926b31fb0bce9b0a2918ad85d274`.
