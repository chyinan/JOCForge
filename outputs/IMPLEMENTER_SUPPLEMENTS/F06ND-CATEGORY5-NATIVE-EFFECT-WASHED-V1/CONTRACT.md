# F06ND category-5 effect boundary

## Approved logical rule

For the selected no-DRC profile, the logical category-5 submission does not write DRC control. The selected state retains four zero FIFO records and override value 0. This is the approved model rule for the selected profile.

## Bounded native process path

The examined selected AtmosProcessor path accepts a per-frame PCM descriptor. It validates frame size and channel count, copies samples into internal storage, calls the processing backend, advances sample cursors, and prepares an output descriptor. The recovered process interface contains no category-5 or InputControlEvent parameter. The selected path does not show a category-5 byte-record write.

A separate generic storage-event API accepts integer/float InputControlEvent vectors, but no source/owner/generation join connects that API to the selected category-5 submission. Its record layout and output effect remain unbound. The generic API is not evidence that category-5 bytes reach BSI or the selected AtmosProcessor.

The selected process path's category-5 byte layout is `NOT_APPLICABLE`. The broader category-5-to-DRC/BSI connection remains `UNBOUND`. The capture was static; no native run or PCM payload read occurred. This bounded path finding is not a global absence claim.
