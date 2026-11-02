# R138 MediaMetadata host dispatch and row writer

## Best-supported model rule

The selected process-state schema creates a `dee::Types::MediaMetadata` value at `+0xA0` under the `dee_audio_source_type` property. The host `OutputWrapper::SetProperty` branch walks its source list and metadata entries, creates typed string-property values, and calls each entry's generic virtual setter slot. The `MediaMetadata` type-specific `set` method uses the same slot offset and turns an ordered vector of string maps into ordered metadata rows:

- preserve input vector order;
- for zero-based input row ordinal `r`, emit `index = r + 1`;
- copy the source filename and nested string key/value fields;
- commit the resulting vector through the SafeType value interface.

The host dispatch and concrete row writer are statically identified separately. Matching property name and slot offset make this the best-supported path candidate, but the generic host vcall's runtime target and receiver pointer are not established. Treat the host→selected R138 `+0xA0` join as `UNBOUND`, not as proof of one exact invocation. The row-preserving model is still ready for a focused input-plumbing trial and does not need to wait for a perfect owner/generation join.

## Keep the typed identity domains separate

For the converter's own typed crosswalk, parse `AO_%04x` suffix minus `0x1001` to obtain zero-based `ConverterChannelInputId`; parse `ATU_%08x` suffix minus `1` to obtain zero-based CHNA UID-table ordinal. The converter verifies equal row counts and same-ordinal typed fields. Do not infer that those IDs equal `MediaMetadata.index` or that the seven-file metadata scan order is the host's runtime source-vector order.

The metadata-only input set covers seven 48 kHz files: six 36-row CHNA files and the one authorized 38-row `人是猫` input (SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`). No payload was read in this slice. A wrong selected metadata row could change filename, sample count, sample rate, offset, or channel metadata consumed by the filter; resulting bitstream effect was not measured.

## Confidence and limit

Confidence is high for the host metadata-property branch, the type-specific row transform, and the converter's internal AO/ATU crosswalk. The exact host generic vcall target, selected R138 receiver pointer, source-vector identity, source order, and generation remain unobserved. No runtime capture, DEE run, PCM read, or encoded-output comparison occurred.

## Exclusions

No implementation source, DEE binary, disassembly, decompilation, audio, PCM, key, or license data is included. The Ghidra report hashes and target image hashes are recorded in `SOURCE-MANIFESTS.json`.
