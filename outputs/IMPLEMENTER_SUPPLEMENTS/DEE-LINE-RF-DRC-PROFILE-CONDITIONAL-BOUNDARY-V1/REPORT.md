# DRC Line/RF profile boundary

## Disposition

This is a reference-only Wash supplement. It does not claim full XML behavior, selected-job delivery, decoded-PCM effect, or implementation readiness.

## Line profile

The reviewed hash-bound static chain maps the six Line XML tokens through a DLL-local profile tree and a separate DDP_COMPR conversion to conditional 3-bit BSI values:

| Line token | Tree U32 | DDP_COMPR U32 | BSI bits if all gates pass |
| --- | ---: | ---: | ---: |
| `film_standard` | 1 | 1 | `001` |
| `film_light` | 0 | 2 | `010` |
| `music_standard` | 3 | 3 | `011` |
| `music_light` | 2 | 4 | `100` |
| `speech` | 4 | 5 | `101` |
| `none` | 5 | 0 | `000` |

The serializer path is conditional. XML delivery to the selected plugin, mode-1 selection, serializer activation, and decoded PCM/audio effect are not observed. `none` is an explicit zero code with presence when the serializer gates pass; it is not the same as omitting the record.

## RF profile

The reviewed packages show `film_light` as a schema default and `music_light` only as an input-only candidate. The selected runtime value, the RF StringType `+0x08` value writer, a typed join from `param_2+0xa4` to the RF descriptor, and any reader/audio/BSI effect remain `UNBOUND` or `NOT_RUN`. The Line mapping must not be copied onto RF.

The package describes parser/native evidence boundaries. It does not authorize implementing DRC audio behavior or claiming XML-to-selected-plugin support.
