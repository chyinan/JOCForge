# Control52 padding write-set status

## Scope

This supplement covers only the statically reviewed main context row table and its full-row copy destination under the selected Control52 writer set. It does not close the row-to-handler identity edge.

Raw row bytes 1–3 are unassigned in the typed field map. For rows written by the Control52 field writer, row offset 0 is stored as a full little-endian DWORD boolean derived from the source U8 at offset 36. That store writes bytes 1–3 as zero. The reviewed arena constructor also zeroes the table before writes, and the reviewed full-row copy preserves all 52 bytes. Under this bounded writer set, bytes 1–3 remain zero in the main context table and its copy destination.

The selector-U16-zero branch is statically reachable and does not write the current row slot; it can reuse an earlier semantic row value. The branch does not establish that it occurs in the selected callback. Do not replace the reused row's semantic fields with defaults or assume the selected path took this branch.

## Remaining identity boundary

The downstream formatter copies raw bytes 0–15 into a staged record beginning at offset +4 and later overwrites only the staged word at +0. A full-U32 gate read at staged +4 therefore spans the row's bytes 0–3. However, the source context table, descriptor copy buffer, formatter output, and selected handler gate table are distinct objects. No same-owner/pointer/generation join proves that the selected gate consumed a row produced by this writer.

Accordingly:

- `CONTROL52_MAIN_CONTEXT_PADDING_1_TO_3 = STATIC_ZERO_WITHIN_REVIEWED_WRITER_SET`
- `SELECTOR_ZERO_SELECTED_CALLBACK_REACHABILITY = NOT_OBSERVED`
- `SELECTED_GATE_LIVE_INPUT_AND_GENERATION_JOIN = UNBOUND`
- `IMPLEMENTATION_READY_FOR_LIVE_GATE_JOIN = NO`

Do not infer a live gate value from row-field arithmetic or call order. Closing the remaining edge requires a Wash-approved, metadata-only receipt with same-row and same-generation identity; no row bytes, PCM, or gate scalar may be read or persisted.
