# Attempt-06 metadata consumer transaction mismatch boundary

Attempt-06 is bound to input and a launch-time executable hash, but the process image/source snapshot identity is not. The run exited 2 at Metadata parent proposal: ParentTransactionMismatch after the meter prepass; no EC3 or partial was present and no expected/actual transaction IDs were logged.

The current source compare point validates the proposal and receipt consumer transaction IDs against the expected source transaction ID before constructing the GTC parent owner receipt. It does not compare the parent callback token at that check. The approved model uses a source transaction token for F05/F13 consumer fields and a distinct parent callback token for the GTC owner receipt. The inspected source path supplies these as separate values.

Therefore the error proves a consumer-token mismatch in the attempt-06 executable, not that the parent callback token was wrong. The current source route is conceptually consistent with the model, but the executable is not joined to that source snapshot, so this is not a runtime fix validation.
