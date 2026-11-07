# RF-mode DRC profile typed snapshot boundary

## Bound XML and typed writer path

The Atmos DDP filter registers `drc:rf_mode_drc_profile` as an `xs:string` with default `film_light` and the six allowed values `film_standard`, `film_light`, `music_standard`, `music_light`, `speech`, and `none`. The bounded RF token path reaches `FUN_180138690`, which writes a generic `StringType` snapshot at the same parser owner base `RSI+0x730`, using the default string on the inspected branch.

## First break

The next conversion or owner edge from that `StringType` snapshot to a numeric RF DRC snapshot field remains `UNBOUND`. The RF path is not joined to DDPI `+0xA4`; the previously examined line-mode `+0xA0` path is distinct and was not reanalyzed here. No DRC audio or BSI effect is established.

## Provenance and limits

Source package `dialogue_rf_drc_profile_typed_path_addendum` seal SHA-256 `e6bf60fe80691ae3daef8544db1caf05b7f1870b1287449d134ed68a7bc043d3`; its normal and `python -O` checker passed. Ghidra 12.2 read-only/no-analysis; installed DLL not rehashed. No DEE/XML runtime, media/PCM/payload/key/license, or implementation access occurred.
