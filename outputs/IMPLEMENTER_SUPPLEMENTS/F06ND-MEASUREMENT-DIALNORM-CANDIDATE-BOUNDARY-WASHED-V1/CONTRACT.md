# Measurement, DRC input, and DialNorm candidate boundary

## Washed result

The hash-identified run-100 configuration does not contain a custom_dialnorm override. Static help describes that option as an optional override in the range -31 through 0, with zero disabling the override. The default or selected branch when the option is absent remains UNBOUND. The ADM GenericSoundEssenceDescriptor_DialNorm field is a separate input-metadata path; no measured-finalize lineage joins it to a DRC value.

Ghidra located a codec-reachable candidate that reads a 16-bit member at offset 1244 and sends the scalar to a 5-bit writer. No semantic field tag, typed owner, generation token, or receipt identifies this candidate as programme DialNorm. It remains a candidate only.

DactMain has a meter-related static path, but this package did not establish a direct DialNorm/DRC/finalize edge there. This is not evidence that such a consumer is absent elsewhere.

## Open chain

- Measurement finalization to the F06ND DRC input: UNBOUND; native execution NOT_RUN.
- F06ND DRC input to the F12 BSI DialNorm field: UNBOUND; the 5-bit candidate is not typed as DialNorm.
- Native scalar owner and source generation: UNBOUND / NOT_RUN.
- The run-100 configuration has no custom override; its default or selected measurement branch is UNBOUND.

No PCM, EC3 payload, license contents, or private keys are included. This supplement is evidence-only and does not change codec algorithms or production readiness.
