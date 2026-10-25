# F06OF producer-to-reader unit boundary

This supplement updates the status of one selected-path dependency. The existing converter-local map ends at `ConverterChannelInputId`; it does not identify an ADM PCM physical row or its measurement-descriptor consumer.

The approved F06OF model measures source extent, callback size, and model cursor in **samples per channel** (`H = 1536`, with the approved `Q + 3` model schedule). A Windows file-read request and result are measured in **bytes**. The reviewed static package has not connected a selected file-position range or Win32 byte count to the selected PCM row, descriptor, or model callback. Do not equate these units or infer a conversion from channel count and sample width alone.

The hash-bound static survey establishes CHNA/ADM mapping validation and duplicate-source-index rejection, and identifies the relevant storage/source type families. It has not located the first selected physical-row write/read or joined it to the selected measurement descriptor. Those metadata checks do not prove `ConverterChannelInputId → PCM physical row`.

Current status:

- `ConverterChannelInputId_TO_PHYSICAL_PCM_ROW = UNBOUND`
- `PHYSICAL_ROW_TO_SELECTED_DESCRIPTOR_OWNER_GENERATION = UNBOUND`
- `READER_BYTE_RANGE_TO_MODEL_SAMPLE_CURSOR_AND_EOF = UNBOUND`
- `F06OF_MODEL_CLOCK = APPROVED_MODEL_SPEC_ONLY`
- `IMPLEMENTATION_READY_FOR_THIS_PRODUCER_JOIN = NO`

The nine input-only cases in the reviewed research package exercise the sample-based model schedule only. They are not native file-read, cursor, EOF, or physical-row observations.

## Minimum identity evidence

The unresolved edge can be closed only by one selected-input/generation receipt that binds:

1. prepared input and source-generation identity;
2. semantic channel ID, CHNA source-channel index, track index, and physical row ordinal;
3. descriptor owner/generation and the row it consumes;
4. file offset and requested/returned read lengths in bytes;
5. source sample cursor before/after and actual samples per channel;
6. explicit EOF and normal/drain callback identity.

These are evidence fields, not an approved product API. Any live observation requires a separate, specific Wash GO; no PCM, row, or payload bytes may be read or persisted.
