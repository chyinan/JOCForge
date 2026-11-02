# Authorized ADM BWF metadata: 36-channel vs 38-channel inputs

Recipient: implementation evidence supplement  
Disposition: the 36-channel file is structurally consistent; this does not establish JOCForge codec admission.

## Sources and identity grades

This package reads only RIFF headers and `fmt `, `chna`, and `axml` chunks. It seeks over each `data` chunk without reading PCM. It does not recompute whole-file hashes. Exact file path, size, and full-file SHA-256 are inherited from a previously sealed authorized-input identity receipt; the receipt package itself and its identity payload were independently hash-verified by Wash.

| Input | Inherited file bytes / SHA-256 | PCM channels | CHNA tracks / UIDs | AXML TrackUID / TrackFormat / ChannelFormat / AudioObject | Block align | Data frames |
|---|---|---:|---:|---:|---:|---:|
| `D:\潜在的なアイ.wav` | 1,069,656,356 / `96e34c167b552928fa9ef287d358c74dac3a2d135a86a0ef4d1529c7d51e8b82` | 36 | 36 / 36 | 36 / 36 / 36 / 27 | 108 bytes | 9,903,478 |
| `D:\人是猫 - 张卡斯, 洛天依.wav` | 743,696,526 / `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1` | 38 | 38 / 38 | 38 / 38 / 38 / 29 | 114 bytes | 6,522,906 |

The current metadata-only pass recomputed CHNA and AXML chunk hashes recorded in `EVIDENCE.json`; it did not read sample data. The inherited whole-file hashes come from `carrier_block_authorized_input_scope_wash_v2`, whose receipt records a prior 1 MiB streaming SHA-256 of each file. That prior receipt was verified as a package, not by re-reading either WAV in this pass.

## What this establishes

The 36-channel file contains 36 CHNA track records and 36 AXML track UIDs; the CHNA record area has the expected 40 bytes per UID. It is a structurally consistent 36-channel ADM/BWF input candidate, not a shortened 38-channel file. The 38-channel comparison similarly has 38 CHNA records/UIDs.

This evidence does not show which rows the selected converter or DDP runtime consumes, nor that JOCForge accepts the 36-channel input end to end. The current reader/profile path still has a 38-channel gate under investigation. AXML `audioObject` element counts and PreparedJobInput bed/object counts are separate typed domains; this package does not equate them.

No DEE run, PCM/data-chunk read, encoded-payload read, or key/license access occurred in this pass.
