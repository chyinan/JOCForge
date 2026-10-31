# Run-100 measurement scalar, encoder, and output-stage observations

## Direct native observations

The hash-matched run-100 DEE reference log records a measurement stage and these scalar values:

- final integrated loudness: -16.784912
- measured loudness: -16.78
- dialogue loudness: -120.00
- ITU loudness: -16.78
- loudness range: 5.73
- sample peak: -4.13
- true peak: -3.95

No units are stated in the log. These are native log observations, not a typed MeasurementReceipt and not a captured F06ND DRC input.

The same log also contains an "Encoder done" marker, 100% encoding progress, and 100% output-stage progress. The controller result records PROCESS_EXIT_OBSERVED and a reference EC3 output file of 17,395,712 bytes. Wash independently confirmed the file exists and has that size without opening or hashing the payload. The numeric DEE exit code is not captured, runtime-loaded image identity is not recorded by the execution XML, and the output bitstream was not validated or listened to. This establishes run100 encoder/output-stage observations only, not product conformance.

## Remaining measurement and DialNorm boundaries

The hash-bound run-100 execution XML contains no custom_dialnorm field. Static help describes the optional override as -31..0 with zero disabling it; the default/selected branch remains UNBOUND. A codec-reachable U16-to-five-bit writer remains a static candidate whose semantic field is not established as DialNorm.

A typed finalization receipt, units, programme/attempt identity, owner generation, F06ND DRC input value, F12 BSI DialNorm value, and same-generation joins remain NOT_RUN or UNBOUND. This evidence does not close the measurement-to-DRC or DRC-to-BSI paths.

No PCM or encoded payload was opened or copied to this supplement. No private key or license contents were read. The separate task-owned scalar observer run remains under research; this package does not change algorithm or production readiness.
