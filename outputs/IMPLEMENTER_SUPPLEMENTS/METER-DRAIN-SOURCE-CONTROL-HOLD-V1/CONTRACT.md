# Meter drain source-control entry

This closes the selected MODEL wiring edge by composing the already approved F01E EOF source-control hold policy with the meter-specific Q+3 zero-raw-input schedule. It does not assert a new native sampler observation.

For normal callback g<Q, call the approved unique F01E normal provider at the actual source start g*1536, strictly below N. The event's own start timestamp remains0. The provider owns sampler state and produces complete control38 and regular37 snapshots using the runtime physical binding, current sampler XYZ/attributes and original audio width. Use those exact current outputs for render/flags/source geometry. The source acceptance domain remains one zero-start block covering N; no new multi-event behavior is defined.

For the last normal callback g=Q-1, actual=N-(Q-1)*1536. Source padding to1536 changes only raw audio. Commit the normal control snapshot under that callback's original owner/attempt/input identity. Keep its full source-control values and the corresponding sampler state/geometry references; do not replace it by target fit/history records.

For each of exactly three meter drains g=Q,Q+1,Q+2:
1. Do not read source audio, look up another ADM event, or invoke a sampler numeric blend. cursor staysN. No t=N or g*1536 input is supplied to the event sampler. Last normal query position is retained only as provenance; it is not the event's start timestamp.
2. Read the last successful normal F01E source snapshot. event_cache, sampler_previous presence/XYZ and source-control snapshot contents are held. Object geometry is the source view derived from that snapshot and runtime route, with the immutable programme bed/basis configuration. Do not add another mutable sampler or duplicate geometry history.
3. Build fresh raw38 positive-zero input, actual0, frontend_samples1536 and a current callback receipt g/g+1 referencing snapshot origin Q-1. Source provenance does not become g merely because the delivery receipt is new.
4. Advance render, audio/flag/feature histories, selection/U04 guard, target assignment, outer matrix, markers/history, component/filter, direct LFE ring and OAR once. Their processing clock and generation continue. The 15 derived target controls delivered by metadata history are current pipeline results, not copies of the source-control snapshot or the previous OAR output. They retain their own producer generation/row.

The fixed-point property is not the reason to skip a sampler blend at EOF: this is the already approved EOF hold branch. A normal callback must still use the approved sampler algorithm; do not replace it with a cached first coordinate because an event is constant.

Evidence: F01E.10 explicitly says to reuse the last complete control snapshot, not advance ADM event time and not reapply an event during drain. F03W stage2 states the same source-control hold. Only the separate main schedule uses four cached-audio drains; meter F06OF.2 replaces audio input and count with three fresh-zero callbacks and continues all downstream state. Its prohibition against copying final SCP/control concerns the derived delivered target controls, not the F01E input snapshot.

The exact frozen meter worker builds controls/geometry before the loop and does not call its object sampler in the drain branch. It creates new zero raw input then calls every listed downstream stage. This proves its fixture path; it is not generic native drain evidence and is not a general normal-callback sampler implementation. The clean adapter above preserves F01E's actual normal provider rather than copying that fixture shortcut.

Existing F01E snapshot and object state fields supply the necessary values. No new DSP state, new coefficient, undefined memory or native input is required. A missing snapshot or mismatched programme/attempt/binding fails before consumption under existing transaction rules; N>0 guarantees at least one normal callback on a valid path. Production numeric execution and generic native sampler drain observation remain NOT_RUN/NOT_OBSERVED.
