# DEE per-source silence record and reader join correction

## Corrected rule

The selected Atmos path statically builds a per-source record with four sample-value vectors: prepend at record `+0x38`, start boundary at `+0x68`, end boundary at `+0x80`, and append at `+0x50`. A later encoder-profile copy places them at `+0x18`, `+0x30`, `+0x48`, and `+0x60`, respectively.

The temporary descriptor built during reader-factory processing is **not** the reader cursor/limit owner. It contains those four values and is used to format a diagnostic string; its vector payload is then moved/released. The reader wrapper is built separately from inline local context fields. The lineage from those fields to the per-source vectors is `UNBOUND`.

This retracts the earlier provisional assumption that per-source prepend/start values directly initialize the reader limit/cursor. In the traced AtmosReader construction path, no consumer of these per-source duration vectors was found that inserts silent samples, extends the read limit, or changes flush count. Other encoder branches or global consumers remain `UNBOUND`; this bounded result does not prove the values are unused everywhere.

## AU-count boundary

The traced output path returns a byte span and writes that span. No AU count, AU-per-span marker, syncframe-size table, or one-AU-per-span rule is bound. A returned byte length is not an AU count.

## Input-only examples and scope

The source package's synthetic vectors are repeated only to document the corrected boundary. They are not DEE executions or media samples. All run-dependent results remain `NOT_RUN`.

- Source package: `dee_atmos_ddp_silence_count_reader_cursor_wash_v1`, MANIFEST SHA-256 `2b5f430ccff98522a432b229dcce77c6667b7c482c49c9f0b9d190a3f21013ea`.
- Wash recomputed all 34 manifest-listed artifact lengths and SHA-256 values; manifest, `.sha256`, and package seal references match.
- The upstream package reports its own normal and `python -O` checker as passed; Wash did not rerun it.
- No source code, Ghidra logs/scripts, PCM, encoded payload, key, or license content is included.
