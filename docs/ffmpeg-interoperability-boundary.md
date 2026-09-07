# FFmpeg interoperability boundary for raw multi-dependent JOC

FFmpeg is an external interoperability validator in this project. Public ETSI
legality, JOCForge structural validation, OpenJOC strict validation, OpenJOC
full render, finite PCM, timing continuity, and deterministic encoding are the
normative/correctness gates.

## Public implementation evidence

The current public [FFmpeg E-AC-3 decoder](https://github.com/FFmpeg/FFmpeg/blob/master/libavcodec/eac3dec.c#L1726-L1756)
documents dependent streams as part of the syntax, then checks `substreamid`.
For nonzero IDs it logs that only ID 0 is decoded, requests a sample for
“Additional substreams”, and returns the unsupported frame-type path. The
public [2018 dependent-frame change](https://ffmpeg.org/pipermail/ffmpeg-cvslog/2018-March/113134.html)
added dependent-frame handling but retained a TODO for additional substreams;
the current source still has the explicit nonzero-ID skip path.

Consequently the measured support boundary is:

| Stream | FFmpeg status | Interpretation |
|---|---|---|
| `I0+D0` | `PASS` | ID-0 independent/dependent path is decoded successfully. |
| `I0+D0+D1+…` | `UNSUPPORTED_DECODER_FEATURE` when independent validation and OpenJOC pass | Additional nonzero dependent IDs are outside the current decoder implementation. |
| Any topology with CRC, impossible size, malformed header, or broken I0/D0 path | `FAIL_STREAM_ERROR` | A real stream error; never relabel it as unsupported feature. |

The local Windows tool used for the evidence run is FFmpeg 8.1. The exact
machine-readable results are recorded by
`scripts/run-multi-dependent-closure.ps1` under the ignored
`artifacts/multi-dependent-closure/` directory.

## Status contract

JOCForge uses an explicit status vocabulary:

- `PASS`: FFmpeg decodes the tested path.
- `PARTIAL`: supported core path is usable, but the complete programme is not
  decoded by the external tool.
- `UNSUPPORTED_DECODER_FEATURE`: failure is causally attributable to the known
  additional-substream limitation after independent structural/OpenJOC gates
  pass.
- `FAIL_STREAM_ERROR`: malformed syntax or a failure in a path FFmpeg is known
  to support.

`UNSUPPORTED_DECODER_FEATURE` never excuses a malformed JOCForge stream. A
failed independent validator or OpenJOC full render keeps the cell failed until
the stream itself is corrected.
