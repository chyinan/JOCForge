# Run36 frame-source descriptor to encoder boundary

Disposition: SAME-INVOCATION_DESCRIPTOR_LINEAGE_OBSERVED; ADM CLOCK AND SOURCE IDENTITY UNBOUND. This is a bounded DEE reference run, not current JOCForge product acceptance.

## Provenance

Source v14 manifest SHA-256: e594526e3599acd7cd2afacf7226f78290a9b40a3e927c72227ca1a15d783619.
NATIVE-FRAME-SOURCE-DESCRIPTOR-CONSUMPTION.json SHA-256: 1c8352df6dd06ff859b94253b10cd3a6eccf797a7d0b0f6b8ba7d308913de2a0.
Run36 job XML SHA-256: 8591f6060d85f8450ee22a94476773388380c1729a5dfab218143f647f210b8a.
Authorized input BWF SHA-256: 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1.
Reference EC3 SHA-256: 077977564a8da534a52dee3feb1c791cc0825dd9cd7eb975575799b02450e686 (262144 bytes).
The source package verifier passed normal and optimized Python; Wash rechecked its payload hashes. No PCM sample values, raw pointers, addresses, disassembly, source code, license, or key are included.

## Same-invocation lineage reported

In one normal reference invocation, the outer source-loop descriptor local was bound to a nested frame-source provider and then to the encoder input by equal live descriptor pointer predicates and same-worker call path. On the captured iteration the reader-relative cursor was 1536, the request was 1536, the span helper returned 1536, and the encoder received 38 channel-config entries, sample span 1536, event count 38. The reader cursor advanced to 3072.

This establishes the descriptor handoff and bounded read span for that invocation. It does not identify the reader's owner, the writer/capacity of the frame pointer table, or the 38 configuration entries as any CHNA UID/physical/global source IDs or selector columns.

## Clock and generation boundary

The reader reports a total extent of 98000 samples; the execution log reports an input interval [4320000,4418000), also 98000 samples. This is numerical equality only; no typed writer/call edge joins reader-relative cursor to ADM sample position or host frame. The cursor interval [1536,3072) is a local reader clock, not a cross-run generation identifier.

No relation to run27, run29, U04, fit generation, or spatial callback generation is claimed. Model comparison is NOT_EVALUATED. Full JOCForge output, product acceptance, and playback remain NOT_RUN.
