# ADM CHNA to physical BWF row order: MVP rule

## Scope and decision

This supplement applies to the authorized ADM BWF metadata/input-row mapping used by the 1024 kbps / 5.1 JOC job. The executable MVP rule is:

1. Build caller `MultiChannelBuffer<int>` rows in physical interleaved BWF slot order.
2. For the seven authorized ADM BWF inputs in the incoming metadata set, map `CHNA.track_index_1based - 1` to the zero-based interleaved input slot.
3. Preserve caller row ordinal through the DEE `DlbWavFile` adapters.

This rule has medium-high combined confidence: the metadata relation matched all 254 CHNA records in seven distinct inputs, and the native row-copy loop preserves caller order. The exact runtime descriptor-to-row-pointer owner/generation join remains `UNBOUND`; under the current MVP acceptance direction it is not a blocker for using the supported row-order rule.

## Hash-bound evidence

Incoming package: `adm_storage_row_table_producer_wash_v1`, source manifest SHA-256 `7fcc5d6b2486c2c882c618843e19ca259e447f2c3ca502eb0688eaec3b5528b0`. The source package verified normal and under `python -O`; all 46 sealed files and the converter-image hash matched.

The static target is the Ghidra 12.2 image `dee_audio_filter_convert_atmos_mezz.dll`, SHA-256 `9972be4f952ca5f9121fa83ef941b29d4c1e6baf0ff1f708baaaedffa1bd76a8`.

- `AdmFileStorage +0x98` lazily stores the `DlbWavFile` source object.
- Parsed `fmt` channel count is read from `DlbWavFile +0xc8`.
- The internal row-pointer table at `DlbWavFile +0xe8` is allocated as `channel_count * 8` bytes.
- The sample-read adapter copies `internal_row[i] = caller_row[i] + sample_offset * 4` for each row ordinal `i`.
- `writeData<int>` row adapters also copy `MultiChannelBuffer<int>` rows by ordinal and apply the sample offset before conversion.
- The traced row writers do not receive or store a CHNA key; the CHNA-to-physical-slot relation comes from the separate authorized-input metadata crosswalk.

## Sample evidence and limits

Seven distinct authorized inputs cover 254 CHNA records: six 36-channel inputs and one 38-channel input. Every record had a unique, complete `track_index_1based` and interleaved slot; every UID/AXML reference resolved; `track_index_1based - 1` matched the interleaved slot in all 254 rows.

The 38-channel `人是猫` ADM identity is SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1` (743,696,526 bytes). The incoming package copied and hashed the complete file to an ASCII alias; no PCM values were decoded, interpreted, or exported. Its metadata-only summary read zero bytes from the `data` payload. The supplement contains no audio payload or absolute input path.

Confidence is high for the seven-input metadata mapping and high for the static ordinal row copy; combined MVP confidence is medium-high because no runtime row-pointer observation or playback was performed. If the combined assumption is wrong, channel/sample content may be routed to the wrong rows; container parseability itself is not affected by this mapping rule. DEE runtime remains `NOT_RUN_SAFETY_GATE_UNPROVEN` because the host's selected file-read callback branch was not shown to avoid key/auth reads. This static/runtime gap does not block implementing the metadata row order.

## Input-only vectors

`INPUT-ONLY-VECTORS.json` records per-input hash, channel/row counts, and the candidate mapping rule only. It contains no sample values, decoded audio, or asserted runtime pointer identities.

## Exclusions

No implementation source, DEE binary, disassembly, decompilation, protected media payload, PCM, private key, or license data is included. The manifest and source table bind the incoming evidence package, image, and metadata observations by hashes.
