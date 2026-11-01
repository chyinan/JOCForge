# F05/F13 attempt -02: bounded runtime timeline

The source record is the stored Codex commandExecution result identified in SOURCE-BINDING.json. It reports the authorized input prepared with SHA-256 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1. The current authorized file was independently rehashed to the same digest; this audit did not reopen or read the media.

The captured timeline shows the Meter prepass completing, callback g=0 committing from phase generation 1 to 2, callback g=1 committing from generation 2 to 3, and then the first F05/F13 body stage failing with Joc(ProducerTransactionMismatch). The inner process exit was 2; Cargo's wrapper exit was 1. The target and its same-name partial were later absent.

The output does not record the expected/actual producer transaction, source token, failing callback token, GTC owner ID, JOC generation, producer/writer generations, source index or dispatch. The word Joc in the error wrapper is not a logged owner ID. Meter metadata_generation=0 belongs to the meter log and is not evidence of JOC owner generation. The failed executable SHA was not captured; the current same-path image hash belongs to rebuilt attempt -03 only.

Use the timeline to bound when failure occurs. It does not identify root cause. Preserve strict validation until the next run records the missing non-payload fields immediately before the comparison. No old partial lacking run identity is reused.
