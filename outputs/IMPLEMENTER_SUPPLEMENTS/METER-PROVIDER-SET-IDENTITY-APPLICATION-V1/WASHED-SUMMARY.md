# Local selected Meter provider identity and aggregate receipt rule

Disposition: LOCAL_IDENTITY_APPLICATION_TO_APPROVED_GTC; NOT DEE NATIVE BEHAVIOR. This resolves the parent-side digest question without extending the audio algorithm or imposing a native key.

## Exact selected provider roster

EXPECTED-MEMBER-ROSTER.json is derived from the hash-bound R3 dependency closure. Include:
- D11 rows classified FRONTEND_FEATURE_PREPASS_CANDIDATE or SPATIAL_RENDER_PREPASS_CANDIDATE (5 + 6 rows);
- selected Meter families D12-R, D12-DV, D12-K, D12-TP, D12-L, F06I, F06L, and F06W.

Exclude the 14 D11 entries classified post-prepass, the selected-but-dead D11-JN branch, and unselected D11-U. The roster records exact approved provider file hashes. This is the expected static member roster, not evidence that providers were loaded or validated.

## Minimal provider-set receipt

For every expected member, the owning provider validator emits a typed immutable member receipt containing member_id, owner_id, approved source/contract hash, validator identity/version, selected profile identity, and a digest of the validated canonical value representation. Encode floating-point values as their canonical word bytes; do not serialize floats as decimal strings. Static rule-only members use the approved rule/contract hash and validator identity.

The parent rejects missing, duplicate, unexpected, or failed member receipts. It sorts receipts by stable ASCII member_id and owner_id, then constructs a versioned ProviderSetPayload with selected profile identity and the sorted member receipts. If the current local API requires the aggregate SHA, compute SHA-256 over deterministic compact serialization of that typed payload: struct field order is fixed, receipt collections use BTreeMap or an explicitly sorted vector, values contain only integers/strings/hashes, and no floats or ambient paths. Recompute the digest in the parent constructor. The digest identifies that validated static provider set; it is not authentication, an audio result, a Meter warm-state hash, or proof of completeness by itself.

Use the roster in EXPECTED-MEMBER-ROSTER.json as the expected membership source. Current provider_set_digest is NOT_COMPUTED because required member validators/runtime identities have not all passed. A digest over the 48-entry index, seven direct assets, test fixture, or cold schemas is invalid as the selected provider set.

## Separate source and runtime identity

MeterSourceOwnerId is a parent-issued ordinary typed local identity, not a DEE token or secret. Bind it through a MeterSourceAdmissionReceipt to programme_id, attempt, PreparedJobInput identity/config, selected profile, validated source frame extent/clock, and the provider-set digest. Keep the same ID through normal callbacks and three fresh-zero Meter drains. On RETIRED/full restart, invalidate the old ID; attempt+1 receives a new root and owner ID. Dynamic OAR/DV32/K/true-peak/energy state and terminal getter values remain owner-generation receipts, separate from the static provider digest.

## Status

The approved selected D11 static consumer set is exhaustively enumerated; 14 post-prepass D11 items are excluded from the Meter roster. Static selected values/rules are identified. Runtime loaders, same-input owner binding, semantic validation for all roster members, Q+3 execution, terminal MeasurementReceipt and READY remain NOT_RUN/ABSENT. No output or algorithm behavior is claimed.
