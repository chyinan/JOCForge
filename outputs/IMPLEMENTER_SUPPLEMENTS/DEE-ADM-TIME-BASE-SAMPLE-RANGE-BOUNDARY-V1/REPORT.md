# DEE time_base control to sample-range boundary

## Bound static behavior

The Atmos DDP job schema and the inspected Priv constructor declare time_base as a StringType with choices file_position and embedded_timecode and a static default candidate file_position. The field is stored at Priv+0x550. The StringType setter writes its string and initialized flag; the matching getter reads it.

The first field-specific read is in setRange. It compares the stored value with embedded_timecode. The embedded-timecode branch applies time offset and range checks while producing start/end seconds. A range converter then checks both bounds against source sample count and rejects end before start; the resulting start/end seconds are multiplied by sampling rate and stored as sample bounds at Priv+0xf98 and Priv+0xfb0. A local map-count guard controls entry to this range block; the alternate path calls a separate helper.

## Open join

This is a static field-to-sample-window rule only. The selected job XML value is not joined to the StringType setter argument or to the Priv instance read by setRange. Runtime owner/generation and branch selection are not captured. No DEE job or audio run occurred. The DLL hash is inherited from the Ghidra project record; the installed DLL was not rehashed.

The upstream evidence checker passed normally and under python -O. This supplement contains no raw listing, native source, XML sample, media, PCM, payload, key, or license.
