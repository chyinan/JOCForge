# Authorized ADM BWF metadata: 36-channel vs 38-channel inputs

Recipient: implementation evidence supplement  
Disposition: metadata-only identity check; not a DEE or JOCForge acceptance result.

## Observed files

Both user-authorized files are RIFF/WAVE PCM24 at 48 kHz and contain `fmt `, `chna`, `axml`, and `data` chunks. The RIFF scanner read only container headers and the metadata chunks; it sought over the `data` chunks without reading PCM and did not compute whole-file hashes.

| Input | PCM channels | CHNA tracks / UIDs | AXML TrackUID / TrackFormat / ChannelFormat / AudioObject | Block align | Data frames |
|---|---:|---:|---:|---:|---:|
| `D:\潜在的なアイ.wav` | 36 | 36 / 36 | 36 / 36 / 36 / 27 | 108 bytes | 9,903,478 |
| `D:\人是猫 - 张卡斯, 洛天依.wav` | 38 | 38 / 38 | 38 / 38 / 38 / 29 | 114 bytes | 6,522,906 |

Each CHNA record area has the expected 40 bytes per UID. The CHNA and AXML metadata-chunk hashes are included in `EVIDENCE.json` and bind these counts without reading sample data.

## Meaning and limits

The 36-channel file is a structurally consistent ADM/BWF candidate, not an incomplete 38-channel file padded or shortened by this check. This supports a real 36-channel admission case for JOCForge. The comparison does not prove the current code accepts it, map a CHNA row into a DEE/JOCForge descriptor, or establish audio behavior. The current implementation report still says the selected profile and reader are fixed at 38 channels; a variable-count input path remains to be implemented and verified.

No DEE run, PCM read, encoded-payload read, or private-key/license access was performed. No whole-file SHA-256 was computed because that would read the `data` chunks. This is a metadata-only result; no PCM/input-only numeric vector is applicable.
