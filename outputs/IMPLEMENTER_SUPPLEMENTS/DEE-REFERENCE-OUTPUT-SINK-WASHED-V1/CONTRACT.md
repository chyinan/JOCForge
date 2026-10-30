# DEE reference EC-3 output sink lineage

## Bounded run result

Run `host_output_sink_capture_v3` completed a full 38-channel selected-fixture reference encode with 4,252 encoder callbacks: 4,247 nonempty 4,096-byte ranges and five empty ranges. Each nonempty range was joined from encoder source through copy destination, FileWriter, WriteFile and final file offset/hash. The 4,247 ranges cover `[0, 17,395,712)` without gaps; one output handle was opened and closed successfully. The recorded final artifact hash is `34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448` (17,395,712 bytes).

MediaInfo CLI 26.05 parsed the recorded artifact as E-AC-3 with JOC, 1,024,000 bit/s CBR, 48 kHz, six channels and 135.904 seconds. This is a DEE reference output observation, not JOCForge output, model/native comparison or audio acceptance.

## Boundary and limits

The host EC-3 stage maps an opaque `stream_object` and invokes the output stage; the static call graph plus same-thread run observation joins this to the DDP output path. Dynamic deep-stack unwind was incomplete. Within the reached DDP/FileWriter/CRT path, the per-range pointer, length, offset and hashes match through the final file. No typed global candidate ID read/join was observed in these traced consumers.

This closes the native reference's final file sink for this run only. It does not prove JOCForge uses the same bytes, nor a general typed host-stream-to-DLL payload identity, other host branches, or native user-music-file output. No raw EC-3, PCM, code or decompilation is included. The final artifact hash was recorded by the source observer; Wash verified the sealed package and range checker, not by re-reading the omitted artifact.
