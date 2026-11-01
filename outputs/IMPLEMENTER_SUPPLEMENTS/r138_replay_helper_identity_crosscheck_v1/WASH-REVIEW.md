# R138-equivalent replay helper identity

One controlled replay used the same hash-identified DEE host image, authorized sample-6 input, and R138 business configuration. Only input/output/temp paths changed. A one-shot observation on the host helper-call site and a child-process-create event occurred in the same debugger session.

The host command referenced a sibling helper in the same installation tree and passed the ASCII sample-6 alias. The child PEB image path class, command line, and loaded PE identity were captured; SHA-256 of the file at the resolved path matched the candidate helper image. DEE exited with code 0. The short temporary EC3 was hashed only and removed without parsing or playback.

This binds the candidate helper image to an R138-equivalent replay for the same host binary/config/input. It does not recover a child PID receipt from the original R138 process instance, so keep that historical child receipt NOT_CAPTURED. Together with the washed sample-6 DBMD tuple, the candidate helper negative-sentinel behavior is now supported in the equivalent host replay. Do not treat the temporary EC3 as a JOCForge result or as listening/conformance acceptance.
