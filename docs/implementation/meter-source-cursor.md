# Independent meter PCM source cursor

Status: `LOCALLY_VERIFIED_SELECTED_RIFF_PCM24_HOPS_AND_THREE_DRAINS__ADM_RENDER_FRONTEND_DEFERRED`.

`MeterPcmSourceCursor::cold` binds a source cursor to an already parsed selected RIFF/PCM24 byte slice, its complete-byte SHA-256, and the selected job's source sample count. The parser computes the hash from the entire borrowed RIFF resource. Cold construction rejects a mismatched digest or sample extent. Every later hop must use the same digest and extent.

Each normal callback reads the next source interval from the borrowed 38-channel, 48 kHz, PCM24 data. A short final raw frame is padded to 1,536 samples per channel with positive zero. Three subsequent drain frames contain zero raw input and do not advance the source cursor. The cursor stores identity and typed clocks; it never retains PCM words. Returned input frames own only the current callback's rows.

The local test builds a small RIFF resource in memory from an input-only recipe. It verifies the final one-sample read, 1,535-word raw padding, three drains, and rejection when either the selected digest or borrowed source bytes differ. No PCM fixture is stored.

This slice stops at physical raw PCM rows. The independent ADM event/render/feature/selection/component producers still need to turn those rows into same-callback SCP16 and control16 values.
