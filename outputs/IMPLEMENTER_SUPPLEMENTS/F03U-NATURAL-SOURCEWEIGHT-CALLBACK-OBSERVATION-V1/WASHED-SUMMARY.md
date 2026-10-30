# Natural source_weight callback and same-thread matrix-view observation

Disposition: ONE_NATURAL_CALLBACK_OBSERVED; FULL_ENCODING_NOT_RUN. This supplements direct synthetic helper probes. It binds one natural callback matrix view to source_weight invocation in the exact target DLL, but does not bind candidate global IDs or establish full U04, codec, or product acceptance.

## Provenance

Source package manifest SHA-256: 58c27ac5c564d94c61bbd185ec4a5a3798c282fcb1dd411db687953456de1e47.
CALL-LINEAGE.json SHA-256: aa8fee99a37a116a1c27af8e166956e1288d95c41bcafa6f7f57f96ac07cf41e.
NATURAL-CALLBACK.json SHA-256: 0d392f50d24a83054e06e740434b98c2db15d271c9f410e6791bd142aa770700.
Target DLL SHA-256: 3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2.
Authorized 38-channel BWF SHA-256: 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1.
Source checker passed normal and optimized Python; Wash rechecked every listed package member hash.

## Same-thread producer and helper join reported

The source package reports IDA instruction-byte matches for the callback wrapper, feature orchestrator, source_weight caller/helper, and local powf in the exact DLL image. In the observed host callback, the callback and feature-producer scopes were active on the source_weight thread, and the caller matrix pointer equaled the helper matrix pointer.

At callback ordinal 30, the feature producer invocation ordinal was also 30; one source_weight caller invocation processed a 24-band x 25-source matrix with source stride 25. The helper made 600 local powf calls, exactly 24 per source. The observer retained only a hash of the 2400 consumed matrix bytes and a hash of the 100 output bytes, not raw values.

This joins that one feature matrix view to that one natural source_weight invocation. It does not identify the 25 source columns as ADM global candidate IDs, prove any earlier PCM-to-feature identity mapping, generalize to other callbacks, or join to the separate U04 geometry projection.

## Runtime/output limits

The approved input BWF was used for the host job. The observation stopped at the first natural source_weight return. A 102400-byte partial EC3 was left in the isolated run directory but was not included or analyzed; full encoding did not complete. Two opaque temporary files were also not included and their payload types remain NOT_EVALUATED. No PCM or feature values were persisted or forwarded.

Candidate global IDs, full selector/projection, U04 numerical replay, later generations, complete E-AC3/JOC output, playback, and product acceptance remain NOT_RUN.
