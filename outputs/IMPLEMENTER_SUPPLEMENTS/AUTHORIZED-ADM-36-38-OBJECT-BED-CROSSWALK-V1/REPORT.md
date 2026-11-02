# ADM 36/38 object and bed metadata crosswalk

Recipient: implementation evidence supplement  
Disposition: the 36-track metadata partition is resolved; exact PreparedJobInput instance identity remains unbound.

## Parsed metadata counts

The R7 Wash-only parser examined only hash-bound `fmt `, `chna`, and `axml` chunks and sought over `data`. The inherited authorized-file receipt supplies full-file identity hashes; no whole-file hash was recomputed in this parse.

| Input | PCM / CHNA rows | AXML `audioObject` elements | DirectSpeakers bed rows | Objects/Objects track rows | PreparedJobInput events |
|---|---:|---:|---:|---:|---:|
| `潜在的なアイ.wav` (36ch) | 36 / 36 | 27 = 1 bed object + 26 Objects objects | 10 | 26 | 26 previously reported; consistent, but exact job/input instance unbound |
| `人是猫 - 张卡斯, 洛天依.wav` (38ch) | 38 / 38 | 29 = 1 bed object + 28 Objects objects | 10 | 28 | 28 expected by the same filter; not re-instantiated in this pass |

Every parsed CHNA row resolved to one AXML UID/track/pack association. The selected graph admits one object event only where both resolved channel and pack types are `Objects`; the ten `DirectSpeakers` rows become bed tracks and are not object events. This explains the 27-vs-26 count for the 36-channel input without equating XML object elements and model object events.

## Limits and use

This supports a variable-row metadata preparation layer: retain all 36 physical rows, create ten bed-track records, and admit 26 object events for this sample. It is not a codec or JOC acceptance result. The precise `PreparedJobInputIdentity`/job XML instance for the earlier 26-event result is absent, and no end-to-end 36-track JOC output has been generated. No PCM, encoded payload, or key was read.
