# F13 preferred typed-result reconciliation

The model-level connection from a completed F06 public measurement result to ID1 preferred remains valid for the approved finite scalar domain. It is an explicit typed path: F06 selector branch -> semantic ProgrammeGatingKind -> PublicMeasurementResultKind -> Id1PreferredValue, where preferred is 1 exactly for public kind Dialogue=1 and 0 for public kinds Ungated=0 or Level=2.

This does not permit a user/XML/UI preferred input, downmix-mode substitution, direct cast of the internal selector-origin code, or direct booleanization of an opaque native I32 provider tag. The semantic enum is not the native tag. The older no mapping status is superseded only for this typed model connection; the native selected branch/value/epoch and full production MeasurementReceipt path remain unbound.

The current source code exposes ProgrammeMeasurementReceipt.public_result_kind() by converting the typed ProgrammeGatingKind. The finalized DI-off helper currently creates only Ungated/Level results; the Dialogue branch is covered by the approved F06 typed selector contract and input-only vectors, but no selected native run/value is established. Thus the mapping rule is ready locally, while the actual selected value/provider and terminal measurement source remain NOT_RUN/UNBOUND.
