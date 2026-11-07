# RF DRC token map boundary

Reference-only Wash supplement. It records a static RF-local token map and the first typed projection candidate. It is not an implementation specification. `IMPLEMENTATION_READY=NO`; do not infer audio or PCM effects.

The sealed source package binds the XML field `drc:rf_mode_drc_profile` as a string property and a static `Priv` layout candidate. Its local table maps six XML tokens to raw 32-bit values written to a per-frame record at `+0xa4`:

| Token | Raw value |
|---|---:|
| `film_standard` | 1 |
| `film_light` | 0 |
| `music_standard` | 3 |
| `music_light` | 2 |
| `speech` | 4 |
| `none` | 5 |

These values are raw table entries, not semantic ordinals or proven DRC profile meanings. The XML/runtime StringType writer, concrete runtime owner/generation, selected provider, downstream consumer of per-frame `+0xa4`, BSI mapping, and audio effect remain `UNBOUND`/`NOT_RUN`. Do not reuse the Line-mode map for RF.

No native source, decompilation, media, PCM, payload, key, or license is included.