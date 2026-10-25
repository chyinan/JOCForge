# Selected PCM input reader static contract

Status: `STATIC_INPUT_READER_SEMANTICS_CLOSED; SELECTED_SOURCE_TO_MEASUREMENT_JOIN_UNBOUND`.

This contract washes the PE-aware static reader erratum and the later successful-frame-return boundary review. It describes the reviewed input-reader path only. It does not prove that the selected measured run used this exact reader, callback table, row storage, or reset generation.

## Units and source layout

The source length and cursor are measured in samples per channel. File-read request and returned counts are bytes. For PCM with `C` channels and `B` bytes per sample, one interleaved sample frame occupies `blockAlign = C * B` bytes. The reviewed PCM24 / 38-channel case has `B=3` and `blockAlign=114`.

The WAVE data descriptor supplies a data-payload file offset and a data length in bytes. For a per-channel sample position `p`, the relative byte offset is `p * blockAlign`, and the absolute file offset is `dataPayloadFileOffset + p * blockAlign`. The actual source file's numeric payload offset and the selected run's descriptor identity remain unbound; do not derive the offset from total file size or PCM size.

## Planar float conversion

On the reviewed planar F32 conversion path, the caller supplies the physical row table. The reader builds temporary row pointers as `callerRow[c] + 4 * destinationOffsetSamples`; the reviewed descriptor uses a one-sample planar stride and F32 representation. Source channel ordinal `c` is written to destination row `c`; this conversion does not apply a semantic channel permutation.

For little-endian signed PCM24, the reviewed conversion sign-extends each 24-bit value, shifts it left by 8 into signed I32, converts to F32, then multiplies by `2^-31`. Algebraically this is `signed24 / 2^23`; exact rounding details beyond the stated F32 operations are not specified by this evidence. The destination rows are zeroed for the requested extent before conversion on the reviewed read-core path.

The reviewed container conversion distinguishes a planar F32 path from another contiguous-row representation. This contract does not claim which variant was used by the selected runtime invocation.

## Actual frame and short-read behavior

For `blockAlign` bytes per sample frame, a chunk containing `actualBytes` contributes `actualFrames = floor(actualBytes / blockAlign)` complete frames. Trailing bytes that do not form a complete interleaved frame do not increase the returned frame count. A short chunk stops the conversion loop. The successful planar-reader return carries the actual frame count through the wrapper; the parent cursor advances by that returned count on the guarded read branch. A separate non-read branch can preserve the requested increment, so the cursor rule must not be generalized beyond the reviewed branch.

When scratch capacity is at least one sample frame, the converter requests whole-frame chunks bounded by that capacity and the remaining request. The smaller-capacity fallback reads per sample/channel and returns only complete frames after a short sample. The exact selected runtime branch and capacity are not bound.

The operating-system read adapter accepts a U32 byte request and widens its returned U32 byte count to U64. The file adapter rejects nonzero status and `actualBytes > requestedBytes`, but accepts a successful short read. In the reviewed cache path, read-ahead may make a lower-level file request larger than one converter request. Internal status observations were success `0`, read failure `3`, and invalid handle `0x11`; these are path-specific status values, not public error names.

No explicit EOF boolean is established. A zero/short read or reaching a duration boundary is not by itself proof of a particular final/drain callback. Do not translate this static reader behavior into a Q+3 native drain claim.

## Descriptor boundary and erratum

The separate measurement-descriptor evidence identifies `channel_count` and `frame_size` fields, but no direct identity join maps this reader's caller-owned row table and actual-frame result to those descriptor fields in the same source and generation. The read interface's caller was not recovered in the reviewed database. The static row-pointer table candidate is not proven to be the selected input's physical PCM rows and must not be promoted as such.

The earlier seek-reader follow-up's `output-only` classification for a shared stream helper is revoked; the helper is also reachable from the input conversion fallback. Earlier single-unwind-region hashes authenticate those regions only, not whole functions. The erratum package provides the corrected complete-body and instruction-slice hashes. Do not infer a full function boundary from one unwind entry.

The unique remaining join is the selected source's actual row storage/read result and returned frame count to the same-generation measurement descriptor, including source/reset-generation identity. Keep it `UNBOUND`; the package establishes static semantics, not selected execution.
