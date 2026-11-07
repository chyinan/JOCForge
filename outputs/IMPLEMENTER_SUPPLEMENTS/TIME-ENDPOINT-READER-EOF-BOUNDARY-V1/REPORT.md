# Time endpoint to reader EOF boundary

## Static range representation

`CompositeMasterItem` stores `startSample` and `endSample`. Construction rejects `endSample > source sample_count` and `startSample > endSample`, but accepts equality and `endSample == sample_count`. The composite builder represents the pair as `startSample` with `length = endSample - startSample`, which is a half-open length encoding.

`FUN_18054a900` transfers start and length to `MasterSourceImpl`; the length getter advances position by that count and creates a tail segment only while the current position is less than the declared total. The tail padding path's `start + length - 1` is a last-index candidate for padding, not evidence of the underlying reader's end predicate.

The bounded `MasterSourceImpl` wrappers prepare `offset + count`, clear `count` samples at `offset`, and pass the request to an indirect storage vtable. Those wrappers do not compare the sample cursor with `endSample`; the storage vtable implementation was not followed in this receipt. Thus the final storage read's termination comparison and actual inclusion of the last sample remain `UNBOUND`.

## Runtime boundary

The existing `human_is_cat` receipt reports two full-file runs but records no selected `CompositeMasterItem`, runtime start/end, storage read request, or EOF predicate. It cannot choose the active reader branch; runtime selection is `NOT_CAPTURED`, audio effect `NOT_RUN`.

## Integrity note

Upstream addendum manifest SHA-256: `1f76b5f2cd2b862fbe16912bbbabe8e30f77755174cc0b8463fcfe5015014420`; upstream seal SHA-256: `5f184ec151d5eff78ca9fae75a4ca1aa179e003ab7c8b1c2b66e370d5a204e29`. The upstream checker passed in normal and `python -O` modes. Its parent reader-input directory currently has six additional unsealed query/trace files; the old seal's originally listed file hashes still match, but the current directory as a whole no longer matches that old artifact set. This new package has its own manifest/seal. It contains no raw traces/scripts, media, PCM, or encoded payload.
