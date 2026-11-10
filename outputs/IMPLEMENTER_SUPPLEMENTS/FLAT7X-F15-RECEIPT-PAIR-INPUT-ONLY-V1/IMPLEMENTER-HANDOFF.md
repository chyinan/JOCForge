---
status: wash-approved-synthetic-input-only
topic: "Flat7x F15 paired F13 and I0/D0 Integrity receipt relations"
spec: "Approved F13/F14K/F15 architecture and typed receipt model"
confidence: contract-vector only; Rust replay not run
tags: [JOCForge, Flat7x, F13, Integrity, F15, receipt, vector]
audience: implementer
---

# Flat7x F15 receipt-pair input-only vector

## Scope

This package gives one synthetic field-level positive pair and rejection vectors for the target-only F15 receipt gate. It contains no frame bytes, EMDF bytes, PCM, media, key, or license. `replay_status=NOT_RUN`; it is a model input contract, not a DEE observation or an audio fixture.

Use it to build a unit-level fixture through the existing F13 metadata and F12/F14K integrity owner APIs. The source-generation values are caller-supplied labels and remain unjoined to any runtime callback/source owner.

## Positive pair

`INPUT-ONLY-VECTORS.json` binds one `IndependentI0` frame of 2560 bytes followed by one `DependentD0` frame of 4096 bytes. The F13 metadata receipt, I0 integrity receipt, and D0 integrity receipt share one synthetic identity: programme, attempt, transaction, source index, dispatch, and `PublishedAuIndex=0`. The parsed EMDF container is associated with D0 carrier block 2 and `C=16` bytes (`C<=511`); its zero-slot and protected digest tokens are explicit synthetic values.

F13 metadata-owner generation, I0 integrity generation, D0 integrity generation, F15 local owner generation, `PublishedAuIndex`, and the `Flat7xSourceGeneration` label remain separate domains. The vector's pair labels match each other; the runtime source/callback generation join remains `UNBOUND`. Frame digest values are to be computed from deterministic synthetic frames generated in memory by a test fixture; no actual frame digest or bytes are asserted here.

## Rejection cases

- Identity mismatch between I0, D0, and F13 rejects the complete AU.
- A carrier block other than 2 rejects the complete AU.
- `C=512` rejects the complete AU without continuation to another field.
- A missing F13, I0, or D0 receipt rejects the complete AU.
- A partial staging receipt (6655 of 6656 bytes) retires the attempt and preserves committed owner state.

## Open gates

This vector does not bind official I0 BSID/BSI, D0 BSI/audio/role ownership, all-AU EMDF fit, the caller's generation to a real source callback, the Flat7x GTC owner projection, or final file exposure. `production_allowed=false` remains in force; the existing 5.1 owner stays separate.
