---
status: partial_implementation_authorized
topic: Flat7x CLI profile selection
confidence: additive control-plane decision
tags: [JOCForge, flat7x, CLI, profile]
audience: implementation
---

# Flat7x CLI selection addendum

This addendum extends the Wash-sealed partial Flat7x profile/header package. It authorizes a selectable, additive route from CLI input to the existing immutable profile-resolution boundary.

## CLI contract

- Add optional argument --joc-layout flat7x.
- When omitted, preserve the existing selected 1024 kbps / 5.1 profile and all current behavior.
- When supplied as flat7x, resolve the existing additive Flat7x profile row and carry JocReconstructionLayout::Flat7x to the existing JOC-header writer.
- Reject unsupported layout values before encoder execution.
- Reject flat7x when combined with a rate or profile outside its approved 1024 kbps row.
- Keep the selector immutable per invocation. Do not introduce mutable global or attempt state.
- Do not add XML, DRC, DI, or timebase options in this slice.

## Boundaries

This is CLI/profile plumbing. It leaves the audio algorithm, ADM row mapping, E-AC-3 channel-map roles, JOC object count, complexity addbsi, EMDF carriage, and AU publication unchanged. Existing mode0/phase1/internal index3 remains separate from JOC reconstruction layout index1.

The addendum authorizes selectable profile resolution only. A generated full EC-3 file, complete official-demo structural match, audio rendering, and listening acceptance remain NOT_RUN.
