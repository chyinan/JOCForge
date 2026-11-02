# DEE encoder-open request/config field boundary

The selected Core/PE provider passes request fields through one builder into a 0x78-byte encoder-open configuration block. One field is explicitly named: an I32 `data_rate` enum in the request is copied to U32 word 10. The mapping from the selected job's 1024 kbps setting to that enum code is not established by this slice.

A separate U16 request field is mode-dependent: in one mode, values 10/12/16 map to internal source-type codes 2/1/0; in the other, the value becomes a source-row count and a pointer supplies six-byte records. An opaque profile vector and nine 0x1c-byte trim records are also copied. None of these fields is named as sample rate, `acmod`, `lfeon`, or wire channel order.

This establishes a concrete request→encoder-open field transfer, but it does not identify missing frame-format values or explain the core PCM mismatch. Do not guess the 1024-kbps enum code from a number or Rust profile. Keep the unbound fields visible while the window-table candidate is tested against audio.

Ghidra 12.2 static only. No DEE run, PCM/payload/key/license read, or output comparison occurred. The source Ghidra script and all raw trace/decompilation are omitted from this Wash supplement.
