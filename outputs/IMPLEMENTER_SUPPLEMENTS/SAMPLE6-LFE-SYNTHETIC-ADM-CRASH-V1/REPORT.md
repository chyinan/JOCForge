# Human Cat synthetic LFE input: DEE crash receipt

Five controlled DEE launches were attempted with a synthetic 38-channel, 48 kHz, PCM24 BWF built from the authorized Human Cat ADM metadata and generated samples only. The first four runs used incomplete or reordered chunk inventories. The fifth run added the source's 64-byte `JUNK` structure and 580-byte `dbmd` chunk; the DBMD bytes were copied without parsing. The source ADM `data` chunk was never read or copied.

All five processes exited with `0xC0000005`; none produced E-AC-3. The final attempt had no matching WER/Application Error record, so faulting module/RVA is `UNOBSERVED`. AU geometry, control-vs-tone output comparison, and LFE transform are `NOT_RUN`/`UNOBSERVED` because there is no output.

This is an input-harness failure receipt, not a codec-conformance result or evidence of the cause of the Rust-vs-DEE PCM residual. No more length/order retries are planned. The synthetic signal, DBMD payload, WAV, encoded output, PCM, keys, and license material are not included.
