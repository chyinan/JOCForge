# 7.1 I0 frame syntax boundary

## Wash-reviewed evidence

The approved official structure summary classifies I0 as `legacy_independent`, but it does not record the exact BSID. The current isolated selected frame-prefix source writes E-AC-3 BSID 16 with `strmtyp=0`. A Wash-reviewed OpenJOC source-capability matrix records that its parser distinguishes legacy AC-3 BSID 0–8 from E-AC-3 BSID 16.

These facts identify a syntax question; they do not prove that the official I0 uses any particular BSID. The official I0 exact BSID remains `NOT_CAPTURED`, and equivalence to the isolated prefix remains `UNBOUND`.

## Implementation boundary

Do not claim that the existing isolated I0 prefix matches the official 7.1 demonstration until an official-only, hash-bound receipt provides its exact syntax. Do not infer the missing value from the label `legacy_independent`, from OpenJOC parser capabilities, or from another output. This package authorizes no code change and does not change the existing 5.1 profile.

`production_allowed=false` remains in force for the new 7.1 profile. The D0 source-role join, native D0 audio algorithm, runtime generation and real 7.1 output remain separately `UNBOUND` / `NOT_RUN`.
