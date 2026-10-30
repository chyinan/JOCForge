# Native-confirmed lowercase drop-frame vector

## Wash result

Run-79 completed a normal DEE reference encode with lowercase `df`, rate token `29.97`, `time_base=file_position`, and tokens `00:01:00:02df` through `00:01:00:12df`. The native composite log records `[2,882,880, 2,898,896)`, exactly matching the hash-bound static prediction. The reference EC-3 is 45,056 bytes. The debugger only launched at its initial break, enabled exception filters, and resumed; it set no code/data breakpoints and read no target memory.

This upgrades that exact lowercase-`df`/29.97/file-position vector from static-only to native-confirmed. It permits implementing this tested branch together with the previously washed static suffix/rate contract. It does not validate semicolon punctuation, uppercase `DF`, other rates, embedded-timecode `df`, invalid label rejection at runtime, output interference, or model/native comparison. Do not claim the whole drop-frame grammar or whole-chain readiness from this one run.

## Evidence limits

The native log, reference output and configuration remain hash-only references; Wash did not include or rehash those source artifacts. The run completed normally and produced output according to the hash-bound receipt. No model output or comparison ran; no raw audio samples were read or persisted.
