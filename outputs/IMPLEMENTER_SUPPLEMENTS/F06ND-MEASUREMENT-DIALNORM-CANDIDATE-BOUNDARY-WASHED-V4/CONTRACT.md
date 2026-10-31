# Run-100 measurement scalars and output-stage boundary

## Direct native observations

The hash-matched run-100 DEE reference log records a measurement stage and these values:

- final integrated loudness: -16.784912
- measured loudness: -16.78
- dialogue loudness: -120.00
- ITU loudness: -16.78
- loudness range: 5.73
- sample peak: -4.13
- true peak: -3.95

No units are stated in the log. The logged measurement values are observed native log scalars, not a typed MeasurementReceipt and not a captured F06ND DRC input.

The same run's log contains an Encoder done line, 100% encoding progress, and 100% output-stage progress. Its controller result says PROCESS_EXIT_OBSERVED and records a reference EC3 output file of 17,395,712 bytes. Wash confirmed the file exists and has that size without opening or hashing the payload. The numeric DEE exit code was not captured, and the output bitstream was not validated or listened to. This establishes output-stage/file creation evidence only, not codec conformance.

## Remaining measurement and DialNorm boundaries

The run-100 execution XML contains no custom_dialnorm field. Static help describes an optional -31..0 override with zero disabling it; the default/selected branch remains UNBOUND. The codec-reachable U16-to-five-bit writer remains a static candidate whose semantic tag is not established as DialNorm.

A typed finalization receipt, units, programme/attempt identity, owner generation, F06ND DRC input value, F12 BSI DialNorm value, and same-generation joins remain NOT_RUN or UNBOUND. The run-100 XML does not bind the runtime-loaded image hashes to the current D-drive files. No full end-to-end or listening result is claimed.

No PCM or encoded payload was opened or copied to this supplement. No private key or license contents were read. This is evidence-only and does not change codec algorithms or production readiness.
