# Wash decision: plan admission and destination capability

## Private plan aggregate

Wash approves a single private, parent-owned `ValidatedEncodingPlan` as the minimum local seam. It may be minted only after the parent binds one actual `PreparedJobInputIdentity` (programme/resource/config/N), the resolver-issued full `ResolvedEncodingProfile`, selected carriage, approved F15 dependency-manifest identity, and owner-issued admissions for the four F15 plan roles: codec/F12, JOC/F05JL, F13/metadata, and F14/integrity. The profile dimensions and selected row come from the hash-bound architecture document listed in SOURCE-MANIFESTS.json.

Each role admission must be produced by its owning validator and retain a reference/handle to the actual loaded immutable value or trusted compiled provider representation, exact provider/asset identity, profile/carriage applicability, and successful validation. A manifest hash alone, empty/caller-created record, fixture receipt, or generic caller-forgeable `passed=true` token is insufficient. The parent aggregates these role-specific receipts privately; it does not create a new public generic validation API.

The 25-entry F15 dependency inventory is not 25 runtime receipts: specs, vectors, review files, context documents, and indexes are not loaded-value evidence. Nested F06/F07/F09/F10 constants remain with their owner validators and stage paths. F06's completed programme `MeasurementReceipt` is separately admitted by the GTC MEASUREMENT transition; per-source analysis, NumericPlan, F05 body/queue, F12 frame, F14 integrity, and durable-append/exposure receipts are created only when those owners actually execute. Never pre-mint these source/runtime receipts during plan construction.

The plan privately derives the canonical F15 OutputProvider projection and SHA from its admitted profile/carriage/dependency inputs; callers cannot supply a projection or hash. It binds checked dynamic Q/source/AU/byte extents, but those counts are not provider hash inputs. OutputOwner construction consumes only this trusted plan, the current parent-owned U64 AttemptId, a distinct opaque staging-owner identity, and a parent-issued validated destination capability.

## Destination capability

The authoritative destination is the existing configured output request passed through the trusted parent/IO boundary; this decision adds no new task/XML/UI preference. Before any staging I/O, the parent validates and returns an opaque private capability bound to the same programme and AttemptId, the exact normalized final-destination identity, the distinct parent-reserved staging namespace, and the existing no-overwrite final-path policy. OutputOwner must reject a missing capability, invalid/conflicting target, or cross-programme/attempt/staging-owner binding before filesystem mutation.

The capability is only pre-I/O admission. It does not certify writes, readback, fsync, rename or final exposure. The IO shell rechecks actual operations and issues durable-append/FinalExposure receipts only after those operations run. This decision adds no symlink, ACL, cross-volume, crash-durability or platform-wide atomicity guarantee beyond already approved IO contracts.

## Status

This is a local typed-authority contract decision based on the approved F15 and output-owner Wash packages, not a DEE native ABI claim. Provider loading, aggregate plan construction, destination validation, OutputOwner construction, READY installation, filesystem operations, and whole-programme execution remain `NOT_RUN`. Keep the production gate closed until those owner paths are implemented and exercised.
