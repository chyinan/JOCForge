# F05 measurement lineage and matched PCM-window boundary

## What is confirmed

The R6 source snapshot at HEAD `6369e49ff518d478913c113b53140b1cade07e1d` has two static typed paths from one finalized `ProgrammeMeasurementReceipt`: `selected_lufs_word` flows into the standard EC3 BSI DialNorm U5 at bit offset 45 through the same-source DRC binding; public result kind, ITU/dialogue loudness and true-peak values flow through ID1 queue receipt/pop validation into the F13 EMDF container. A same-run receipt-to-output byte observation is still missing. Measurement receipt, DRC, metadata queue, source, dispatch and AU state retain distinct owner/transaction clocks.

Implementation HEAD advanced to `2f23885008cd789a5b17b639a1a3737b2ad87273`; Wash independently rehashed all 17 R6-listed Rust source files against that HEAD and all 17 matched. The static typed paths therefore remain current for those files.

## Audio comparison already available

The prior matched full-file pair binds the same authorized ADM and job XML and contains 4,247 AUs per output. In the washed decode window `[2750,2875)` (192,000 frames per channel), decoded channel 3 has RMSE about `0.00548` at DRC scale 1 and `0.00545` at scale 0; other five channels are below `0.0005` at scale 0. This is close but not identical in that window. It is not full-track PCM equality, Atmos renderer parity, or listening verification; the channel-3 identity/cause is still unbound.

A later Rust file passed full decode-only validation but has no fresh DEE comparison and came from an older source commit. The user clarified that artifact `593b…c0473` was the initial structural-error experiment and was fixed later; it is historical-only, not a current blocker. The newer candidate's exact input/XML/run join is still handled by the current implementation evidence task.

## Next minimal closure

Record exact input/XML/executable/output hashes for the current JOCForge run, capture one same-run F06/F13 receipt-to-output record, and compare the identified output against the matching DEE reference. Keep the channel-3 residual as the leading candidate to investigate; do not claim its cause before the matched current output is measured.

No new encoder, decoder, PCM capture, playback, or private-key operation was performed for this supplement. It contains only washed metrics and hashes, no audio payload.
