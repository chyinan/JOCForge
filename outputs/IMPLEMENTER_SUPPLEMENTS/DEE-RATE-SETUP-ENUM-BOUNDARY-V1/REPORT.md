# DEE data-rate enumeration to typed setup boundary

## Six static values

The inherited static selector/setup path maps the six XML rate texts to these candidate setup values when each value reaches the lookup and same setup path:

| XML text | Selector code | Typed setup value |
|---:|---:|---:|
| 384 | 0 | U32 `param_1[2] = 6` |
| 448 | 1 | U32 `param_1[2] = 8` |
| 576 | 2 | U32 `param_1[2] = 8` |
| 640 | 3 | U32 `param_1[2] = 8` |
| 768 | 4 | U32 `param_1[2] = 8` |
| 1024 | 5 | U32 `param_1[2] = 8` |

The static setup route is `FUN_1801a7d50 → FUN_1801abc10 → FUN_1801ac130`; `FUN_1801b2250` constructs the typed codec-configuration candidate.

## Still open

This does not prove that every text is selected at runtime or that the typed setup field is the E-AC-3 BSI bitrate. The rate-derived config at `piVar17+0x206` remains unjoined to writer state `param_1+0x810`; the concrete dynamic writer target, writer owner/generation and BSI field remain `UNBOUND`. Runtime and encoded output were `NOT_RUN`.

## Provenance

This Wash supplement consolidates only the hash-bound two-value and four-value static addenda listed in `SOURCE-MANIFESTS.json`. Both upstream package checkers passed normally and under `python -O`; no DEE run, media/PCM/payload/key/license access, implementation-source access, or process control occurred.
