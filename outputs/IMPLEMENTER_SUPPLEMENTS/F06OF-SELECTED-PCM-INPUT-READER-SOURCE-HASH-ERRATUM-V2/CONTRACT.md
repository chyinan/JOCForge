# F06OF reader source-hash correction

Status: `V1_ERRATUM_WITHDRAWN; UPSTREAM_SOURCE_IDENTITY_MATCHES_APPROVED_MODEL`.

Withdraw and disregard `F06OF-SELECTED-PCM-INPUT-READER-SOURCE-HASH-ERRATUM-V1` (manifest SHA-256 `93ba9c3e308567e001656738b84eb5069573a155ee39c9fc6e00f6bf6d61f7ba`). Its attribution was wrong: the mismatched digest was not present in Agent 4's original reader source-identity package. That package's `source_identities.json` matches its own manifest and records the correct F06OF model-spec hash.

Wash independently verified the original `f06of_seek_reader_unit_followup_wash` package and compared its F06OF spec reference to the approved package manifest and file bytes. All three hashes match at `2a98742b9032b900bb42da7f6a7c404a835c9539fbb55040939d7d025c8ce423`. The selected-input reader static contract's source reference was also already correct.

The incorrect digest was introduced by Wash in an undelivered draft of `F06OF-READER-HOST-ADAPTER-STATIC-CONTRACT-V1`; that draft has been corrected before handoff and now references the approved digest. This is a Wash transcription correction, not an upstream source mismatch.

This correction changes no reader algorithm or native evidence grade. The physical row/read-result to same-generation measurement descriptor join and EOF/drain behavior remain `UNBOUND`.
