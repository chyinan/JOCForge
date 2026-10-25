# F06ND public distribution scope

This version supersedes V1 for the selected F06ND material. It binds both the 13-word functional provider and the separate 34-word mathematical provider used by the selected normalizer/dialnorm/quantizer leaves. It does not change or replace any hash-sealed source package.

## Authorized public material

The user has clarified that non-secret algorithm constants and washed implementation material may be public. Under that direction, public distribution is authorized for:

- The 13 binary32 values in `DRC_FILTER5`, `DRC_NORMALIZE3`, `DRC_QUANTIZE3`, and `DRC_AUX_ROTATION2`.
- The 34-word mathematical provider used by the selected normalizer and dialnorm mapping: 31 derived binary32 dialnorm words and 3 mix words. Its explicit formulas are independently generated mathematical rules; comparison against source behavior is validation only, and no original table is included.
- The selected none-DRC algorithm specification, normalizer, dialnorm-offset and RF/line quantizer implementations, generated input vectors, tests, validation results, and non-sensitive provenance.

The five `DRC_FILTER5` values are non-secret audio coefficients classified as historical DEE static functional audio evidence. The other eight functional values are binary32 literals from verified frozen Python functional consumers. The 34-word math provider is derived from its stated selected-domain mathematical formulas. These are ordinary non-secret technical parameters, not private keys or authentication material.

## Exclusions

This authorization does not include vendor private keys, authentication secrets, genuinely private user data, raw PCM/audio inputs, protected media outputs, original vendor tables, source code, binaries, or research artifacts. No such material is part of the public-scope supplements themselves.

Earlier internal-only and no-public-distribution labels for the material covered here were agent-added and are superseded by the user's clarification in this thread. The older source package and its hashes remain unchanged as historical provenance; this addendum supplies the current distribution decision.
