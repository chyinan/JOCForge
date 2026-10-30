# Plan-bound OutputOwner admission crosswalk

## Accepted local boundary

The washed F15 contracts require a trusted, parent-owned `ValidatedEncodingPlan`; `PreparedJobInput` plus `ResolvedEncodingProfile` alone is not output authority. The plan binds the same programme, resource, config and positive N; the full resolved profile; selected carriage; approved F15 dependency identity; owner-validated selected provider material; and the private, canonical F15 output-provider projection. The caller cannot supply an independent F15 provider or provider hash.

Plan admission and runtime source-stage receipts are distinct. Plan admission must establish that the selected provider values were actually loaded and accepted by their owning validators for this profile/carriage and input identity. A dependency-manifest hash, spec entry, vector or review document alone is not proof that a runtime parameter value was loaded. Do not mint a generic caller-forgeable PASS receipt. Keep actual stage receipts (F05/F12/F13/F14 and other source-stage results) owned by their stages and create them only when those stages run.

`OutputOwner` construction is a parent-only operation consuming the trusted plan, the current typed U64 `AttemptId`, a distinct opaque staging-owner identity and a destination that has passed the selected output policy. It derives the provider projection/hash internally, recomputes state identity from actual canonical bytes and rejects cross-programme/input/profile/carriage/attempt/owner/destination bindings before I/O. The GTC parent performs the one atomic READY swap using actual after-states; this contract does not claim the constructor or swap has run.

The selected architecture document does enumerate the six `EncodingProfile` dimensions and selected row: `AuthoredInputProfile` (selected ADM BWF, 38-channel PCM24, 48 kHz); `CoreCarrierProfile` (5.1, six carriers); `JocDownmixProfile` (mode 0, phase 1, index 3); `SubstreamTopology` (one independent, zero dependent); `CodecRateProfile` (1024 kbps, six blocks, 1536 samples/AU, 4096 bytes/AU); and `EmdfCarriageProfile` (`deployed_compat` or `etsi_emdf`). `ValidationProfile.render_target` is separate. See the hash-bound architecture source in SOURCE-MANIFESTS.json.

The user-approved local GTC v3 availability contract remains authoritative for the local projection: METERING has initialized meter plus eleven UNINITIALIZED owners with no fabricated hashes/refs; READY installs the eleven actual cold roots and retains terminal meter. Approved v2 is unchanged. The accompanying v3 approval-status erratum supersedes stale proposal annotations without mutating the sealed v3 schema.

## Not closed by this crosswalk

The exact runtime provider-role closure, each role's owner-issued typed loaded-value receipt/handle, and the complete destination-capability producer remain under Wash investigation. The selected dependency inventory has mixed specs, parameter assets, indexes, vectors and context items; do not treat all entries as runtime receipts. No plan provider loader, destination validation, OutputOwner construction, production root creation, READY install, filesystem I/O or end-to-end programme was run. Preserve `NOT_RUN` for those operations.
