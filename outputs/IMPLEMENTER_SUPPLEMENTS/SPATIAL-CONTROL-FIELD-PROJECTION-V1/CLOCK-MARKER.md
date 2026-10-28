# Spatial control normal clock and marker after-state

## Normal callbacks

F01E/F03W define `Q=ceil(N/1536)` as the planned number of normal source callbacks. The F01E cold constructor and F03W state schema contain a distinct U64 `normal_callbacks` field initialized to zero. GTC `eof_ledger.normal_callbacks` is a cumulative committed ledger, monotonic within an attempt; `main_total_callbacks=normal_callbacks+main_cached_callbacks`. The selected example has terminal Q=4247, not a universal constant.

LOCAL_MODEL proposal for the F03W cold-zero field: on a successful NORMAL parent commit increment it once; on each CACHED_DRAIN keep it unchanged; failure keeps before value; full restart reconstructs zero. After all normal callbacks it equals Q. This is the natural count implied by its cold state and the schedule, but the approved sources do not explicitly identify the F03W stored field with GTC's ledger field or state a direct per-callback assignment. Treat that identity and transition as a proposed local clarification for Wash review, not an observed native or independently proven cross-owner join. Keep planned Q separate from committed count and apply checked U64 arithmetic.

F01E's sample input-only clock case supplies N=6522906, frame=1536, 28 object inputs, and four cached drains. Its output assertions are not runtime seeds. F03W's synthetic N=4072 schedule supplies three NORMAL callbacks (including a short final read) followed by four CACHED_DRAIN callbacks. These fixtures exercise schedule semantics; they do not independently prove a v5 stored-field mapping or real ADM execution.

## previous_marker_active

F05H.2-.3 is the marker producer. For each of 15 target slots it reads assignment current-active and the same owner's before `previous_active`. Quiet is determined from the current active-slot/matrix/flags witness: absent a positive coefficient on an eligible source, force current-active to zero. After marker selection, `next.previous_active = current_active_after_quiet` and `next.current_active` has the same per-slot value. This update is per slot, including zero-valued inactive/quiet slots; it is not the I32 marker value, initial marker list, or previous target XYZ/ID/attribute.

At callback0, previous-active is the declared U8[15] all-zero cold value. At every subsequent successful NORMAL or CACHED_DRAIN callback, the F05H marker transition computes the next 15 values from that callback's actual assignment and quiet witness; cached input/control does not freeze target/marker state. For zero selected targets, F03G installation preserves its prior current/previous/active fields, but the subsequent marker still executes and can clear active through quiet. Failure discards the proposal and leaves committed previous-active unchanged; reset restores the declared cold array.

F05RT uses its state-owned marker-after as the next callback's previous active, and commits its history/previous-active/current target snapshot only after the complete callback succeeds. F03W owner projection lists `previous_marker_active`; F05R56 separately owns marker/history state. The approved texts describe one semantic transition but do not provide a separate v5 storage/receipt mapping that proves two independently mutable copies are identical. A single owner with an explicit parent-admitted projection is a LOCAL_MODEL integration decision; do not create a second independently advanced copy or infer an identity from equal arrays. The frozen standalone front prediction model reads its `previous_marker_active` and stores marker `previous_active_after` in its next state, corroborating only that model instance.

## Scope

Input-only cases in this package contain inputs and transition discriminators, no expected output state. The approved source vectors may contain separate expected assertions; they must never seed after-state. No native transaction, pointer identity, real ADM callback, listening or whole-product validation is claimed. F05H/F05RT marker semantics are specified for their selected input domain; generic layouts and unsupported branches remain outside scope.
