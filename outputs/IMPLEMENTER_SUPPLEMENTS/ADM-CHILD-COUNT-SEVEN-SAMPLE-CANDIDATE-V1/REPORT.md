# Seven-ADM child-count candidate validation

## Result

Across the seven authorized ADM BWF metadata receipts, every DirectSpeakers
bed row resolves to one AXML channel reference, so each sample has a ten-row
bed-vector sum of 10. Counting CHNA rows whose `pack_type` is `Objects` gives
26 Objects for six samples and 28 for `人是猫`. The metadata candidate
`Objects rows + sum(bed vector lengths)` therefore yields 36 for six samples
and 38 for `人是猫`; all seven results match their CHNA declared track counts.

| Authorized ADM | Objects rows | Bed vector lengths | Candidate | CHNA tracks |
|---|---:|---|---:|---:|
| First Love | 26 | ten rows of 1 | 36 | 36 |
| Hortus Regis Alti | 26 | ten rows of 1 | 36 | 36 |
| My jealousy (Original ver.) | 26 | ten rows of 1 | 36 | 36 |
| アルジャーノン | 26 | ten rows of 1 | 36 | 36 |
| 潜在的なアイ | 26 | ten rows of 1 | 36 | 36 |
| 人是猫 - 张卡斯, 洛天依 | 28 | ten rows of 1 | 38 | 38 |
| 咲き誇れ - YOASOBI | 26 | ten rows of 1 | 36 | 36 |

The AXML object-node count is one greater than the CHNA `Objects` track-row
count because the AXML object count includes the bed object. It is not the
Object term used by the candidate formula.

## Disposition

This is strong seven-sample metadata support for the candidate formula; it
is useful for a best-effort implementation trial. It is not a native runtime
measurement: the original WAV/BWF files were not opened in this slice, native
bed-row identity was not captured, and the selected child-count mode and
U16 remain `UNBOUND`. Do not claim that a runtime getter returned 36 or 38
from this metadata-only evidence.

## Scope

The package reuses hash-bound washed AXML/CHNA metadata and the static native
scalar formula. No PCM/data chunk, media payload, private key, or license
material was read; DEE was not run. The full per-row UIDs, resolved channel
IDs, chunk hashes, and sample-hash claims remain in the source package and
are hash-bound by its manifest.
