# Generalized JOC space closure investigation

Verified 2026-09-07 against JOCForge HEAD `98a8fe117f6f3211c792db8cb9d695afefb6de11`, ETSI TS 103 420 V1.2.1, ETSI TS 102 366 V1.4.1, and the local public OpenJOC source/behavior.

## Current implementation

- `JocProfile` has only `Idx0`, `Idx1`, and `Idx2`.
- `profile_spec` already distinguishes the five-channel, Flat-7.X rear pair, and 5.X+2 height pair. D0 source indices are profile-owned.
- `write_joc_payload` and the source-derived package path already use the profile's downmix index and input width, so idx3/idx4 can reuse the existing source-derived five/seven-channel paths once the public profile enum/spec mapping is extended.
- The CLI accepts only numeric idx0/idx1/idx2 names. I0 skip-field carriage is deliberately rejected for profiles other than idx0.
- `rate_budget_for_carriage` is carriage-scoped rather than profile-scoped. Existing strict evidence covers 1024, 1280, 1408, 1536, and 1664 kbps for idx0–idx2; lower paired rates reject on D0 capacity.

## Public closure

- ETSI TS 103 420 V1.2.1 Table 47 defines idx0 `5.X`, idx1 `7.X` with `L,R,C,Ls,Rs,Lb,Rb`, idx2 `5.X+2` with `L,R,C,Ls,Rs,Tfl,Tfr`, idx3 `5.X with 90 degree phase shift`, and idx4 `5.X+2 with 90 degree phase shift`; idx5–idx7 are reserved. Table 48 gives 5, 7, 7, 5, and 7 JOC input channels respectively.
- The same document states that LFE is optional in these lists and bypassed rather than processed by the JOC decoder. JOCForge's LFE core path must remain outside the reconstruction matrix.
- Clauses 8.1–8.3 require an EMDF container, put it in the last dependent when dependents exist, require payload IDs 11 and 14, and define the strict payload configuration and addbsi complexity field.
- Annex E restricts CMAF to one I0 plus optional D0, six blocks per frame, and 1,536 samples per sample. This is not a general raw elementary-stream restriction.
- TS 102 366 Table E.1.3 defines one, two, three, or six blocks per syncframe. Dependent substreams use sequential IDs and custom channel maps; the standard's general substream rules must not be inferred from CMAF limits.
- Public OpenJOC tests confirm idx3/idx4 phase profiles are signaling-only at the JOC stage; they do not apply a second PCM phase rotation. OpenJOC also independently validates idx0–idx4 topology identities, general D0..D7 assembly, short-block grouping, and the original-syntax AC-3 I0 plus E-AC-3 D0 boundary.

## Safe implementation boundary

The first implementation slice should add semantic names and a central immutable profile/carriage/rate plan, extend idx3/idx4 as signaling-only profiles, and generate a multidimensional capability lattice. General short-block, multi-dependent, CMAF, and legacy-core generation remain explicit `SPEC_LEGAL_NOT_IMPLEMENTED` or out-of-scope cells until JOCForge has independent writer and full validation evidence.
