# F05R56 mode-2 current/previous direct-stage contract

Status: `DIRECT_STAGE_ONLY`; `PRODUCT_INTEGRATION=NOT_RUN`; `RAW_SNAPSHOT_ADMISSION=BLOCKED`.

This supplement records the selected normal mode-2 spatial direct stage. It is
an implementation input contract for the isolated direct receiver/record
builder only. It does not define a public raw-snapshot validator, a programme
transaction, or error rollback semantics.

## Input shape

- `frame_samples`: `3072`.
- `source_count`: `48` source records.
- `target_count`: `15` output records.
- `current_xyz`: 15 rows, three F32 words per row, row-major.
- `current_active`: capacity 16, with the selected normal fixture active rows
  `0..=5`; the output record count remains 15.
- `previous_xyz` and `previous_active` are owner state supplied by the direct
  stage. A caller must not provide a second conflicting previous snapshot.

## Record projection

Each output record is 56 bytes (14 U32 words). The three words at byte offset
24 (words 6..8) are copied byte-for-byte from the corresponding row of
`current_xyz`. The source is the current array; it is not the previous/history
array. The remaining record fields are outside this supplement and must remain
under their existing approved contract.

## Normal callback state update

Before the selected mode-2 call returns successfully, the owner copies the
15x3 `current_xyz` array into `previous_xyz` and the 16-entry
`current_active` array into `previous_active`. This is an observed normal-call
in-place update. It is not evidence of an external proposal/receipt or of
rollback after an exception.

## Explicit limits

This supplement does not define:

- raw snapshot finite/range admission or inactive-row rules;
- error precedence, exception translation, or rollback;
- programme, attempt, transaction, profile, or callback-generation identity;
- publication, codec, parent integration, or whole-programme acceptance.

Those boundaries remain fail-closed and must not be filled by inference from
this normal-call observation.
