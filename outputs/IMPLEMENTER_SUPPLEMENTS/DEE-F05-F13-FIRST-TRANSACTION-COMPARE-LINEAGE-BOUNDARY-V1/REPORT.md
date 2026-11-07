# F05/F13 first producer-transaction comparison boundary

Wash verified the sanitized source package manifest, payload hashes, and selected working-tree file hashes. The inspected source is a hash-bound working-tree snapshot; Git HEAD and failed-executable identity were unavailable, so this is not a claim about the historical binary.

The first application comparison in the inspected snapshot is `metadata_source_bridge.rs:274`: actual is `joc_after.id14_body().provenance().producer_transaction_id()`, expected is `Some(&source_transaction_id)`. On the explicit source route, `source_execution.rs` constructs the expected source token from validated `CodecSourceIndex`; the same typed token is passed into the transaction-aware source stage and forwarded unchanged through `JocPayloadInput`, proposal, representation, body proposal, and provenance. Thus no expected/actual producer-token divergence is observed before this comparison on this current route.

Generation is a separate typed lineage. The mapping `source_index = callback_index - 2` and writer-generation checks occur independently; generation is not an operand in the first producer-transaction comparison. The parent callback/GTC token is also a distinct argument and is not an operand of that comparison.

The historical failure receipt lacks expected/actual tokens, parent token, generations, and failed executable hash. Therefore the historical first diverging operand and cause remain `UNBOUND`. Do not treat this current-source result as proof that the historical failure is fixed, and do not loosen the validation based on this package. No tests, DEE, media, PCM, payload, key/license access, or implementation change occurred.
