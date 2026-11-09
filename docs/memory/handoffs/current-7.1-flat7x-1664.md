# Current 7.1 flat7x / 1664 handoff

## Target

The approved target has separate I0 and D0 frames: I0 at 640 kbps, then D0 at 1024 kbps. Their 1664 kbps total is two frames in one access unit, not one 1664 kbps frame. The AU target requires same-source-generation validation and complete publication; an I0-only partial result is rejected.

## Verified in the isolated checkout

- The profile target, rate/frame-size arithmetic, AU composition requirement, and field-local chanmap U16 serialization each have separate sealed Wash supplements. Their identities and exact remaining boundaries are in [evidence-index.md](../shared/evidence-index.md).
- The target chanmap field rule is numeric only; semantic speaker labels come from the separate profile contract.
- The prior partial native transaction package does not prove I0/D0 writer identity, target order, same-generation lineage, or atomic publication.

## Open boundaries

- D0 source-row-to-semantic-role mapping: `UNBOUND`.
- Native D0 audio algorithm: `UNBOUND`.
- Runtime source-generation join: `NOT_CAPTURED`.
- Legacy durable atomic publication: unproven.
- Real output and W04 acceptance: `NOT_RUN`.
- `production_allowed=false`.

## Next action

Wait for sanitized Wash material that closes the D0 source-role and processing contracts. Until then, do not infer those rules from chanmap set bits or from the partial legacy transaction package, and do not enable production.
