# Run37 source-storage sample-count validation boundary

Disposition: SOURCE_ITEM_SAMPLE_COUNT_OBSERVED; RUN38 MAIN READER JOIN WITHDRAWN. This corrects the earlier cross-path interpretation of the 6522906 value.

## Provenance and verification

Source v17 manifest SHA-256: 22f36b20684eecb0f3b058c5bd36fc7ca97f2a2df3b2a6c94cfc8cab0c70d665.
Capture payload SHA-256: 10a508fc57a98ea34ccfc560373162aa2e55095d6ca44ec52fdd730703ff3ae2.
Run37 job XML SHA-256: a0c30b0b71efe7348ba307f1028855b0ed3a1c39d7d2855392269946ffa6622b.
Authorized input BWF SHA-256: 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1.
The source checker passed normal and optimized Python; Wash rechecked all package member hashes. No raw sample values, source, decompilation, address, license, or key are included.

## Source-item range validation

The static caller path reports that a per-source item opens an Atmos storage object, calls a generic sample-count getter, and validates its range. If end_sample is zero, the returned sample count is substituted as end_sample. Otherwise end_sample must not exceed the returned count; start_sample must not exceed end_sample.

In run37, the getter returned 6522906. The source-storage object's path identity was not captured. The source package explicitly states this object is not the same reader object as run38's main reader extent writer. The run38 main reader separately stores 98000 from its own duration accessor. The v15 join of 6522906 to that main reader is withdrawn.

Run37's reference encode completed with the established 262144-byte output hash, but this is not the current JOCForge product output. No full model comparison, reader-clock join, source-ID join, U04, Gate1, or product acceptance is claimed.
