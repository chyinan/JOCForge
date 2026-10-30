# R33 spatial source and selector evidence supplement

This v2 supplement supersedes the earlier staged capture whose manifest SHA-256 was `f899782becbfbdbcc598f6196f6fa9deb56e706976eabd51e5c81eb9fbf32a67`. It repeats the owner writer, 48-source descriptor, selector, assignment, final target map, and matrix consumer in a single native invocation.

## Scope

This supplement binds one authorized 38-track ADM-BWF metadata identity to one completed licensed DEE reference run. It reports one native callback generation from owner-row construction through candidate gating, 36-to-15 selection, 15-slot assignment, final target map, and the following 15-by-48 source-matrix consumer shape.

The input AXML and CHNA SHA-256 values exactly match the approved input-only F03R crosswalk. In that same native invocation, the 37-row owner state table is the suffix beginning after the explicit 11-entry prefix in the 48-entry source pointer table. The 37-row geometry table is the corresponding suffix of the 48-row descriptor table. Both suffixes were bytewise equal at relative offsets 11 times 8 bytes and 11 times 56 bytes.

For this exact fixture, owner row r maps by same-generation pointer identity to source descriptor column r+11. Under the matched F03R input crosswalk, regular columns 11 through 47 map to global source IDs 11 through 47. Prefix columns 0 through 10 are aggregate provenance and have no ADM AO UID.

## Gate and selection vector

The native source descriptor has 48 rows. Its candidate feature input has 24 bands with stride 48. The gate reads the active flag at row offset 4 and includes a row when the flag is zero and the ordered binary32 sum of the 24 bands is at least 0.5. The same-call input-only vector records every per-row sum, flag, and gate result; the 36-row native candidate list matches that formula.

The selection helper returned 15 unique candidate rows. A 15-element assignment permutation maps those selected rows into the final target-slot order. All 15 target slots were active. The next matrix consumer used the same 48-source pointer table with a 15-by-48 descriptor; numeric coefficients were not captured.

## Limits

This is a fixture- and profile-scoped lineage result. It does not establish the selector's complete general rule across arbitrary layouts. The full helper branch/threshold rules, numeric source matrix, U04 numeric call/output, independent model replay, Type 10 composition, and six-plane comparison remain NOT_RUN or NOT_CAPTURED. The reference EC-3 hash identifies DEE output only; no model output was produced.

This Wash package contains no source audio, raw PCM, native pointer address, module name, decompilation, or private key/provider material. It is not an implementer package and does not contact the implementation agent.

