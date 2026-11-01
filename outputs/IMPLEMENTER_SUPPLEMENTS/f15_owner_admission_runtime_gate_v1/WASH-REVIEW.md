# Wash review: F15 production admission inventory

This is a hash-bound audit of the current isolated implementation sources against the approved F15 contract. It supplies no audio algorithm and invents no provider value.

**Four provider-role values are not available in production code yet.** `F15RoleValidatedValue<Role>` is an empty marker. There are no production implementations of it and no owning F12/F05JL/F13/F14 validator entrypoints returning such immutable values. The only implementations are `TestProviderValue` in the test module, which is marked for all four roles. `F15ProviderAdmissionCandidates` and `ValidatedEncodingPlan` are generic scaffolding; `admit_validated_encoding_plan` has no production callsite and its only invocation is through a test adapter.

**No trusted destination capability issuer exists in production yet.** The private `ParentValidatedDestinationCapability::from_parent_validation` checks parent IDs, final identity, staging namespace, and `NoOverwrite`; current callsites are test-only. `ProgrammeOutputBinding::for_test_from_prepared_input` is gated by `cfg(test)`/`test-support`. `OutputStagingOwnerId::new` only wraps a nonempty string; it is not a parent/IO validator and cannot mint publication authority. The approved F15 spec assigns staging/output-ledger ownership to F15, but runtime construction must come from the real trusted parent/IO boundary.

What exists: `F15PlanParentContext::from_prepared_input` captures programme ID, prepared-input identity, selected/resolved profile, selected dependency-manifest identity, and source sample count. It does not supply the four role-specific values or a destination capability.

At the inspected HEAD and hashed working-tree sources, `jocforge_application::encode()` still returns `BackendUnavailable`; it reaches no product input, codec, or output IO. Therefore this inventory identifies concrete production wiring work; it is not a successful output receipt. Do not replace the absent owner values with booleans, manifest-only PASS receipts, or the test provider.
