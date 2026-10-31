# Embedded timecode to EndSample truncation residual

## Washed result

This is a bounded result for the numeric frame-index timecode path in R128/R129, using the same authorized ADM BWF, DEE executable, and relevant DLL hashes. The two runs are separate process executions and differ only in the end-frame token: 86664 versus 86665; the start token is 86640.

The hash-matched Ghidra report and setter-linked pair support this sequence for this path:

1. The end frame token is treated as an inclusive frame token and incremented before conversion to an absolute time boundary.
2. The absolute boundary is converted to an absolute sample boundary by rounding at the 48 kHz sample rate.
3. The automatic source-time offset is subtracted to form relative seconds.
4. The relative seconds are multiplied by the sample rate and converted to a signed integer, truncating toward zero.

For end token 86664, the absolute end boundary is 173,330,000 samples; the relative floating-point product is 49,999.999999992724 and the EndSample setter receives 49,999. For end token 86665, the absolute boundary is 173,332,000 samples; the relative product is 52,000.000000007276 and the setter receives 52,000. Thus the absolute boundary delta is 2,000 samples while the relative setter delta is 2,001. Setter-call identifiers join these values to the same-run CompositeMasterItem fields and reader observations.

This localizes the observed one-sample residual to floating-point truncation in the post-conversion relative setter for this hash-matched path. It does not change the separate R121 finding that EndSample itself is an exclusive upper sample boundary.

## Execution and limits

Both controllers returned zero, both native process exits were observed, both logs recorded encoder completion and 100% progress, and both reference outputs were nonempty. The numeric DEE process exit code was not captured by the controller. Reference output hashes/sizes were recorded but no output payload was included or listened to; this is not a product-conformance or listening result.

The source-origin metadata (24 fps, 48 kHz, 3,610 seconds) is carried from a prior metadata-only package for the same BWF. That package says the metadata provenance was transcribed from an earlier native observation; its raw debugger transcript was not persisted. Offset-versus-FFOA identity remains UNBOUND. R115-to-R128 execution identity and sample-to-spatial-generation remain UNBOUND.

The Ghidra report used for this conversion pair is included by path and SHA-256 in the source manifest and its bytes were independently rehashed by Wash. This does not reconcile the separate old Wash V2 report-hash metadata discrepancy for the R121 field-semantics package. No PCM, EC3 payload, private key, or license content is included.
