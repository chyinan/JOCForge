# JOCForge current Rust LFE path

## Wash disposition

The hash-checked source audit shows that the current Rust candidate path does more than route an LFE row: it has LFE-specific post-filtering and a dedicated seven-bin analysis/encoding path. This is a description of current source, not a DEE parity result. No DEE LFE coefficients or complete algorithm have been independently compared in this package.

## Current source path

1. The ADM graph uses CHNA UID and format references to determine physical host rows. The bed join identifies the unique LFE by `channelFormatID`, `semantic_role == lfe`, and `bed_id == 3`; regular bed/object controls omit that host.
2. The LFE source row is converted to finite F32 and serialized to little-endian bytes. A three-slot callback ring selects source callback `g-2`; the Type10 assembler places that row into row 0 without a filter at this boundary.
3. In the spatial receiver, lane 3 receives a four-section LFE post-filter. All six lanes also pass through shared preconditioning, accumulation, descriptor gain, and the post-S09 delay/scale stage.
4. MAIN analysis copies `[L,R,C,LFE,Ls,Rs]` into six F32 analysis rows. F07 history includes LFE; F07A transient processing excludes it. F08 applies the shared window fold and a dedicated LFE transform to seven bins. F09/F10 build and pack the seven LFE exponents.
5. The selected candidate-frame path retains seven LFE coefficients/exponents, BAP-quantizes the coefficients, and writes LFE syntax with `acmod=7` and `lfeon=1`.

## Product and comparison limits

The public `encode()` path currently returns `SourcePipelineUnavailable` before starting the source audio pipeline. The candidate transform/frame path therefore does not establish that a playable current-product EC-3 was generated. The user's report about the damaged test file is not bound to a run receipt or this HEAD. The previous Python comparator is described as a wrapper around an opaque external codec; a same-length/different-hash comparison did not isolate an LFE cause. Accordingly, this package proves neither that JOCForge matches DEE nor where they differ.

The upstream audit bound HEAD `1b4a34170bc7981f2c330308a1f1b4c642258c24`, 24 current source hashes, 13 tracked dirty-source hashes, and seven approved Wash package hashes. It reports untracked workspace files present but not enumerated. No PCM, EC-3 payload, DEE run, DSP execution, or key/license access occurred.
