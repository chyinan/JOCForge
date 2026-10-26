# F06ND category5 selected-path status

## Selected-profile rule

For the approved selected profile, the opaque category5 / 32-bit / count-1 submission does not write the DRC-control FIFO. The four zero logical records remain zero and the selected override remains zero. This is the approved logical FIFO contract; it does not define a byte-level record representation.

The statically reviewed selected `AtmosProcessor::process` path receives a PCM frame descriptor (channel count, frame size, per-channel sample pointers). The nonzero-frame path copies samples, calls the Atmos backend, advances sample cursors, and prepares an output descriptor. This process interface has no category5 or `InputControlEvent` input. A category5 byte-record layout is therefore `NOT_APPLICABLE` at this selected process boundary.

## Scope limit

A separate generic storage API accepts `InputControlEvent` vectors, but no producer/owner/generation join connects the selected F06ND category5 submission to that API or to selected BSI/output. Its contents and effect remain `UNBOUND`. This supplement does not claim that the entire native engine has no generic event records; it only says no category5 byte schema is needed on the observed selected process path.

No native execution was performed. No PCM, payload, key, source code, or decompilation is included. Do not add a byte-level FIFO schema for this selected path unless a joined producer/consumer edge is later established.
