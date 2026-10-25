# Control52 formatter to selected gate row join

## Static relation on the reviewed serial path

Within one reviewed serial frame owner, the row arena base is `B`, the constructor prefix has `P` rows, the current invocation formats `C` rows, and the current-row capacity is `K` with the precondition `C <= K`. Each row has a 56-byte stride.

The arena layout is:

- combined gate rows: `[B, B + 56*(P+K))`;
- prefix rows: `[B, B + 56*P)`;
- current-row suffix: `[B + 56*P, B + 56*(P+K))`.

The formatter writes current row `i` at `B + 56*P + 56*i`. The parent passes the same combined base `B` and total count `P+C` to the selected gate. Therefore the static row index is `gate_row = P+i`. This closes the containing-table and current-row suffix pointer relation for the reviewed serial call path; it corrects the earlier inference that different starting addresses alone implied different arenas.

## Exact limits

This is pointer/slot identity, not value equality. Reset and active-state update functions can mutate row fields between formatting and gate consumption; no gate scalar or numeric membership parity is established. Actual selected callback invocation, live `P/C/K`, buffer capacity, and source-generation token remain unobserved. The source-context `+96` table from the separate 52-byte source-row path is another owner and is not this frame gate arena.

Status:

- `FORMATTER_TO_GATE_ROW_SUFFIX = STATIC_CLOSED_ON_REVIEWED_SERIAL_PATH`
- `GATE_ROW_INDEX = P + i`
- `RUNTIME_CAPACITY_AND_SELECTED_INVOCATION = NOT_CAPTURED`
- `ROW_VALUE_EQUIVALENCE = NOT_EVALUATED`
- `SOURCE_CONTEXT_TO_FRAME_GATE_GENERATION = UNBOUND`

No dynamic capture is authorized by this supplement. A one-parent, one-row metadata-only confirmation requires a separate Wash GO and may not read row bytes, PCM, or gate scalars.
