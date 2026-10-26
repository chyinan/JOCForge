# Additional-BSI complexity index decoder-consumer status

Status: `OPENJOC_ADMISSION_GATE_STATICALLY_CLOSED; DIRECT_RENDER_PARAMETER_NOT_FOUND_IN_SCOPED_PATH`.

The E-AC-3 additional-BSI type-A complexity value is an eight-bit field. In the reviewed OpenJOC path, the parser requires the type-A flag and valid reserved bits, reads the U8, and rejects values above 16. The access-unit/session layer compares the parsed value with OAMD's total object count before invoking the JOC payload decoder. A missing/malformed field or a mismatch prevents that frame from reaching decode/render.

For a valid matching field, OpenJOC retains the value for diagnostics but does not place it in the `JocFrameInput` or decoded-frame structure. The checked reconstruction path consumes JOC/OAMD payloads, decoded rows, PCM/LFE, sample rate, and frame index. Therefore this field acts as an admission/consistency gate in the reviewed path, not as a direct spatial reconstruction parameter. This does not imply that changing OAMD or other JOC payload fields would preserve output.

The LAV source path parses the U8 and writes it into MOV `dec3` container metadata. A LAV playback decoder/render consumer was not established in the bounded C/C++ identifier scan; this is not a whole-repository absence claim.

## Scope and limits

This is a static source audit only. It did not load or decode audio, run DEE/DLL, or execute tests. Eleven listed source/design file hashes were independently rechecked. The report's repository HEAD and clean-worktree claims were not independently checked because Git metadata access to those repository roots was denied; the file-level digests are the identity basis for this supplement.

The audit does not prove that the selected native DEE producer's configuration count joins to the in-band field in the same source generation, does not establish audible severity, and does not replace product listening acceptance.
