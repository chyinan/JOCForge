# F03G target-slot installation

This mapping covers only the local F03G.2-F03G.4 transition over explicit old
and selected values. The whole F03G path and product integration remain
deferred.

## Inputs and validation

The call receives 15 old slots with current XYZ words, IDs, attributes,
previous XYZ/ID/attribute values, and 0/1 active flags. It also receives
same-generation selected XYZ/ID/attribute arrays with a shared length from 0
through 15. Current and selected XYZ must be finite F32 words. Selected
attribute multiplicity must not exceed its count across all old slots,
including inactive slots. No first-install exception is defined.

## Cost and assignment

For each old row and selected column, distance starts at positive zero and
processes X/Y/Z in order using separately-rounded F32 subtraction, multiply,
and addition. Cost adds the R32 ID mismatch penalty 20, then the R32 attribute
mismatch penalty 100, then positive zero. The returned original costs are
row-major. Every cost must be finite and strictly below the largest finite
F32 word.

For nonempty selections, mapping starts at -1 and performs exactly one greedy
minimum-cost choice per selected column. Each pass scans old rows then new
columns, replaces the current choice only for a strictly smaller cost, and
masks the chosen row and column with the largest finite F32 word. This
preserves earliest row/column tie behavior and is not a Hungarian assignment.

## State projection

An empty selection returns an all -1 mapping, no costs, and the entire prior
state unchanged, including active flags. For nonempty selections, unmatched
slots become inactive while retaining all other fields. Matched slots become
active, copy current fields into previous fields, and install the mapped new
XYZ/ID/attribute values. The function owns no persistent state and does not
perform cold construction, source gating, selection, projection, marker/history
updates, or parent commit.

The implementation is bound to four source cases in the selected package:
`U-FRONT-INSTALL-EMPTY`, `U-FRONT-INSTALL-RECT`, `U-FRONT-INSTALL-TIES`, and
`U-FRONT-INSTALL-ATTR`. The first three compare mapping, original costs,
branch selection, and all current/previous/active state fields. The attribute
case verifies multiplicity rejection.

## Remaining boundary

The local result is not production-usable F03G. Source gate, cold provider,
selector/projection/fit producers, marker/history, callback parent state,
transactional commit, real ADM, and whole-programme acceptance remain
deferred or `NOT_RUN`.
