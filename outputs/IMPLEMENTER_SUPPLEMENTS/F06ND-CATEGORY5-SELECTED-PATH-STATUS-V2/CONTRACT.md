# F06ND category5 selected-path boundary

The approved selected-profile clean rule remains: the opaque category5 / 32-bit / count-1 submission does not write the DRC-control FIFO; the four zero logical records remain zero and the selected override is zero.

The additional static evidence is limited to the selected `AtmosProcessor::process` boundary. That path receives a PCM frame descriptor, processes per-channel samples, and has no category5 or `InputControlEvent` parameter. A byte-record layout is therefore not applicable at this boundary.

A separate generic storage API accepts `InputControlEvent` vectors. No selected category5 producer, owner, or generation has been joined to that API, to extra metadata, or to selected BSI/output. Keep those possible cross-boundary effects `UNBOUND`; do not infer a global absence or invent a category5 record layout.

No native execution occurred. This supplement adds no new implementation algorithm; it narrows which selected boundary would require a byte layout and preserves the remaining exact join gap.
