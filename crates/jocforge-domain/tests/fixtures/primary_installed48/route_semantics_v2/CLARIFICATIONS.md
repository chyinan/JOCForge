# Primary route semantics clarification

This addendum supersedes `PRIMARY-ROUTE-SEMANTICS-CLARIFICATION-V1` for the three points below. It supplements, and does not alter unrelated rules in, `PRIMARY-INSTALLED48-ACTIVITY-V1`.

## Width bindings

Follow `F04G-WIDTH-SEMANTICS-ERRATUM-V1` (manifest SHA-256 `a77333ef0916eca7e739c9b12c2d6c87e8dbe1441e5d0a017e286f2772ffea8d`): F04G.5 runtime width descriptor and F04G.6 power/edge-weight operands use `mapped_width`; cell kernels use selected `width_grid` values; F04G.7 sin² and front gain compensation use `raw_width`.

## Quiet-route field 44

For a quiet or nonpositive-width route, clear byte offset 40 only. Preserve byte offset 44 from the source row. In the bound example, source index 22 starts with U32 offset 44 = 1, and quiet variant 45 retains U32 offset 44 = 1 while offset 40 is 0. Do not zero offset 44 on this route.

## Active width at or above 0.2

For an active regular row (`quiet=0`) with positive width, first clear U32 offsets 36, 40 and 44 as specified. When the pre-route width is `>= R32(0.2)`, also set U32 offsets 48 and 52 to zero and set the Z position at byte offset 32 to positive F32 zero. This is the direct-component disappearance branch; do not preserve the source Z position in this branch. U32 flags at offsets 0 and 4 are both set to 1.

For source index 22 / physical host 12 / semantic object `AO_100D`, source Z at offset 32 is F32 `0.1` and source width at offset 40 is F32 `0.20000001788139343`, which is `>= R32(0.2)` (`0.20000000298023224`). The matching active variant 44 therefore has offsets 0/4 = 1/1, offsets 32/36/40/44/48/52 = 0, and condition `quiet=0`. The matching quiet variant 45 has offset 32 unchanged at F32 `0.1`, offset 40 = 0, offset 44 = 1, and condition `quiet=1`.

The field offsets above are byte offsets in the 56-byte little-endian record. Do not treat array element indices in `metadata_words_8_to_55` as byte offsets: that array starts at byte offset 8, so its element 6 is byte offset 32 and element 9 is byte offset 44.

## Validation boundary

The source row and both variants are sealed model-derived artifacts whose hashes are listed in `PROVENANCE.json`. Their decoded values agree with the washed route rules and threshold branch. This does not claim native primary acceptance, end-to-end encode, or product acceptance; those remain `NOT_RUN`.
