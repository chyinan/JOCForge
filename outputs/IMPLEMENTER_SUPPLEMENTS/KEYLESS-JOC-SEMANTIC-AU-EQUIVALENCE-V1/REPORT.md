# Keyless JOC semantic access-unit equivalence

## Result

OpenJOC 0.20.1 inspected the user-designated protected DEE reference and keyless comparator with the same full-stream command (`inspect <input.ec3> --json --aus --objects`). Both traversals exited 0 and parsed 4,247 AUs. The per-AU semantic digest matched with zero AU mismatches.

Both inputs expose JOC payload ID 14 in all 4,247 AUs, the same payload order, carrier location (`I0 / audio_block_skipfld`), 15 coded objects, 15 reconstruction rows, complexity index 16, 16 OAMD slots / 15 dynamic slots, and 67,952 metadata updates. Parser diagnostics were zero; timing was continuous. Both pass OpenJOC deployed compatibility and fail ETSI Strict on the same seven signaling deviations.

## Limits

- **Verified:** parser-visible AU and JOC/OAMD semantic summaries match for these two user-designated streams.
- **Not verified:** coefficient-value vectors, rendered coefficients, JOC object rendering, or Atmos spatial-render equivalence. OpenJOC reports no render verification and exposes no coefficient vectors in the inspected JSON contract.
- **Not verified:** current Rust output, which was not included in this comparison and is separately user-reported as unreadable.
- **Identity scope:** the protected/reference file hashes are inherited from prior Wash receipts; the keyless candidate hash is inherited from the prior Wash comparison. The R7 inspection did not recompute media hashes or save raw parser JSON/payloads.

Use this as a validation target for future output structure and metadata semantics, not as an encoder algorithm source. Do not treat a parser pass as proof of audio or spatial-render equality.
