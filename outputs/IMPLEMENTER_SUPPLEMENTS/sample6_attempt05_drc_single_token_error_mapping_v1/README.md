# Attempt-05 DRC single-token error mapping boundary

Attempt-05 terminated with exit code 2 at DRC parent proposal: ParentTransactionMismatch; no output or partial was present. Its input and run ID are bound, but the executable actually loaded and runner path at launch are not.

Static review of the current DRC API shows the legacy single-token wrapper passes one token as both source and parent. If the inner source-token comparison fails, it maps SourceTransactionMismatch to ParentTransactionMismatch. A two-token API separately accepts the source transaction and parent callback transaction. Current source uses that API, but its file modification time is after attempt-05 launch and no process-image/source join exists. Therefore this is a static explanation consistent with the error, not a recovered attempt-05 runtime cause or value.
