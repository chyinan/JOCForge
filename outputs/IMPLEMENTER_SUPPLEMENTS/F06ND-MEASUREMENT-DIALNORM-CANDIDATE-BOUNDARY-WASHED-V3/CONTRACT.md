# Run-100 measurement scalar log boundary

## Newly verified native log fact

The hash-identified run-100 DEE reference log contains a measurement-complete entry followed by these scalar values:

- final integrated loudness: -16.784912
- measured loudness: -16.78
- dialogue loudness: -120.00
- ITU loudness: -16.78
- loudness range: 5.73
- sample peak: -4.13
- true peak: -3.95

The log does not state units for these numbers. Wash independently rehashed the exact run-100 log and matched it to the source identity. This is a native log observation, not a typed MeasurementReceipt, not a same-programme/attempt/generation receipt, and not a captured F06ND DRC input.

## Existing static boundaries

The hash-bound run-100 execution XML has no custom_dialnorm field. Ghidra help describes custom_dialnorm as an optional -31..0 override with zero disabling it; the default/selected branch is still UNBOUND. ADM GenericSoundEssenceDescriptor_DialNorm is a separate input-metadata route.

A codec-reachable U16 member-to-5-bit writer path remains a static candidate. It is not semantically identified as programme DialNorm, and no typed owner/generation joins it to the run-100 log values.

## Open chain

Measurement scalar values are now OBSERVED_LOG_ONLY for run-100. The typed measurement/finalize owner and units remain NOT_RUN/UNBOUND. Measurement finalization to the F06ND DRC input remains UNBOUND; F06ND DRC input to F12 BSI DialNorm remains UNBOUND. Playback and full end-to-end output validation remain NOT_RUN.

No PCM, EC3 payload, license content, or private key is included. This evidence does not change the codec algorithm or production readiness.
