# Dialogue Intelligence key-to-Bool owner boundary

## Bound static lookup

Priv+0xec0 is a ParametersContainer whose vtable slot +0x38 performs exact candidate-name lookup. It compares bytes over the shorter length and requires equal lengths; no case folding or normalization was observed. Zero matches throw not-found, multiple matches throw ambiguous-property, and one match returns that candidate object pointer.

The Priv constructor builds the candidate vector with the dialogue_intelligence Bool at Priv+0x290 and passes that vector to the same ParametersContainer. The Bool constructor registers the exact key loudness:measure_only:dialogue_intelligence once and stores its name vector. The Bool vtable +0x18 accessor returns that vector. Therefore a lookup using that exact key selects the unique registered Bool candidate in this constructor-built container.

## Runtime boundary

No runtime receipt shows that the dispatcher actually requests this key or returns this candidate in a selected job. Runtime owner/generation and any measurement, audio, or JOC effect remain UNBOUND/NOT_IDENTIFIED. DEE/UI/runtime were not run.

The upstream package manifest SHA-256 is f2192a5956fab3a03be35d944ac1b28c767b9bf59f41e29674a911193d01d326 and the package-seal file SHA-256 is 86edc665ba5725e3848b67e503489ce0c8fcb8d0a225cf94360d3cd64a447c35. Its 51-file checker passed normally and under python -O. No raw Ghidra material, media, PCM, payload, key, or license is included.
