# Conditional PCM24 intermediate observation

## Finding

A hash-bound Ghidra static package reduces one conditional three-byte BWF read branch to a signed 16-bit intermediate. Its static algebra is consistent with signed PCM24 divided by 256, rounded to nearest with ties to even, with a positive-endpoint guard.

This is only a conditional code-branch fact. The selected ADM profile has not been joined to this branch at runtime, and the intermediate has not been joined to the first numeric F32 DSP consumer.

## Wash disposition

This package is not an implementation rule for the selected ADM-to-JOC path. Do not change JOCForge PCM24 input handling or F32 conversion from this observation alone. The approved PCM24 input contract remains the implementation source until a same-generation runtime observation closes the S16-to-F32 path.

The separate four-byte float path is not joined to this three-byte branch. F32 normalization, precision, rounding and first numeric consumer remain open.

## Provenance

Source package manifest SHA-256: d9ae53629c9093a048286d2b64add3936ced350fd488e876029ccb78fe0b3031
Source package seal SHA-256: 7b066da9a392069c20cbdfc8d07cfa6117c4dced74a62f5abe16c6aabe8b0511

The source package checker passed in normal and optimized Python modes. This Wash package contains no source code, disassembly, binary, PCM payload, private key, or absolute source path.
