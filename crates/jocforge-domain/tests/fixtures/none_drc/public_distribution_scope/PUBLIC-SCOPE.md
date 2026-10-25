# F06ND public distribution scope

This addendum supersedes the earlier internal-only distribution labels for the selected none-DRC functional constants and their clean implementation artifacts. It does not modify or replace any hash-sealed source package.

## Authorized public material

The user has clarified that ordinary non-secret algorithm constants and washed implementation material may be public. Under that instruction, the following selected material is authorized for public distribution:

- The 13 binary32 functional values in `DRC_FILTER5`, `DRC_NORMALIZE3`, `DRC_QUANTIZE3`, and `DRC_AUX_ROTATION2`.
- The frozen none-DRC algorithm specification, selected RF/line scalar quantizer implementation, generated input vectors, and verification results that use those values.
- Provenance and source-classification metadata needed to identify which values came from historical static audio evidence and which came from the frozen Python model.

The five `DRC_FILTER5` values are non-secret audio coefficients classified as historical DEE static functional audio evidence. The other eight values are binary32 literals from verified frozen Python functional consumers. These are algorithm parameters; this package contains no vendor key, authentication secret, personal data, raw PCM, original table file, source code, DLL, or protected media.

## Exclusions and scope

This authorization applies only to the listed non-secret F06ND functional values and their clean specification, code, and generated validation artifacts. It does not authorize publication of vendor private keys, authentication material, raw user/private data, PCM/audio inputs, protected outputs, original vendor binaries, or original source/research material.

The earlier `INTERNAL_RESEARCH_AND_INTERNAL_IMPLEMENTATION_ONLY` and `public_or_third_party_distribution_authorized=false` labels were agent-added restrictions and are superseded for this material by the user's clarification in this thread. The original package remains sealed and unchanged; consumers should apply this addendum when determining the current distribution scope.
