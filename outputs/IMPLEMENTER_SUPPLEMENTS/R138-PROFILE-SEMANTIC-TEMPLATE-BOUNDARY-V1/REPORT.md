# R138 profile semantic-template boundary

## Configuration evidence

The original R138 execution XML has SHA-256 `fa65143f…d1018a4` and declares input `audio.adm`, output route `encode_to_atmos_ddp`, 1024 kbps, `embedded_timecode` window 86640–86664, meter mode 1770-4, DI off, threshold 100, DRC none, four downmix levels -3 with preferred mode `loro`, and surround/height trims `auto`.

A completed First Love job XML has a different SHA-256 (`a04b7881…7211cb`), but its normalized tree digest matches the R138 source after redacting only `file_name`/`path` text (`0b319d7e…e78b8726a4`). This proves equality of the recorded non-path XML configuration in that comparison, not input-media identity. The First Love receipt marks its short window quiet; it is a configuration template, not an active-audio result for “人是猫”.

Hortus and My Jealousy active-window XMLs use 86664–86688 (+24) and are per-input observations, not defaults. Run-100 is file-position based and not an equivalent timing profile. The local XSD is not hash-bound to the executing DEE image and rejects the R138 element order; do not rewrite the original profile to satisfy that unrelated local schema.

## Disposition

Sanitized receipts alone do not provide new run-instance input/output/temp paths. A separate completed First Love XML template exists, so a new *derived* job could preserve its non-path settings while changing only run-specific paths to the authorized same-hash ADM alias and unique output/temp locations. Such a file would have a new hash and is not the original R138 XML. This is a bounded candidate for a metadata-only pre-Run observation; it is not created or run in this Wash package.

No raw XML, media, PCM, payload, or license/key content is included. No new DEE job was started.
