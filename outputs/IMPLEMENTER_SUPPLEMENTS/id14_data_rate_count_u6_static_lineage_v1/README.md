# ID14 data_rate selector and native U6 path: static scope

This package independently washes the R2 Ghidra 12.2 static follow-up. It identifies a plugin-owned Priv.data_rate field of type xs:list:string, its rate-string-to-enum table, and the normal-path ID14 count construction through the U6 writer.

The data_rate string 1024 maps to enum 5, which is accepted by the selected conversion path and belongs to the nonzero selector branch. In that normal branch, code13=15+marker_count and code14=marker_count, so the count seed is 15; the native writer serializes count-1 as U6, yielding 14. For enum 0 (384), the normal base is 11. These are static model vectors, not observations of a particular job.

The host task's data_rate being applied to this exact Priv instance and its packet are not dynamically joined. The six-byte packet row names remain unrecovered. The frozen model's per-frame EMDF parser and object_count+1 BSI calculation are a separate path with no same-file/job token to this native parameter. Do not infer identity from both paths producing ID14-related counts; do not add a user-facing preferred boolean.

Special mode is separate: code13 is an adjusted source count, code14 is 1, and its difference is adjusted_count-1. No special-mode behavior is generalized into the normal data-rate path.

Only sanitized contracts, source hashes, and static input-only vectors are included. No Ghidra decompilation, native source, media, PCM, payload, bitstream, or private key/license content is included.
