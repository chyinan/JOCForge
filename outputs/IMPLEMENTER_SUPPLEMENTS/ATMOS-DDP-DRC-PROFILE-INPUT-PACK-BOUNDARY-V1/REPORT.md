# Atmos DDP DRC profile input and packed-control boundary

## Bound native profile map

For both Line and RF profile StringType parameters, the DDP plugin validates six strings and uses a static string-to-U32 map:

| Profile string | Native U32 |
|---|---:|
| `film_light` | 0 |
| `film_standard` | 1 |
| `music_light` | 2 |
| `music_standard` | 3 |
| `speech` | 4 |
| `none` | 5 |

The two StringType values are stored at backend `+0x6d0` (Line) and `+0x750` (RF). The encoder packer converts the values into workflow-block slots `+0xa0` and `+0xa4` respectively. In the non-mode-1/2 path, that block is passed through a wrapper to `FUN_180175c80`, which performs shape/length validation.

## Defaults and XML handoff are distinct

- DDP StringType constructor defaults: `film_light` for both fields.
- Host DocModel template defaults: `music_light` for both fields.
- Local XSD defaults: `film_light`, but XSD identity is not bound to the executing DEE.
- R138 source config: explicitly supplies `none` for both; that source XML identity is not joined to the executing input.

Do not infer which default wins or that the R138 values reach the native fields. The XML child/value to StageRunner `+0x68` row, and conversion of unprefixed host field names into DDP `drc:` keys, remain `UNBOUND`.

## Final consumer/effect boundary

The static scan found no direct read of packed workflow slots `+0xa0/+0xa4` in the bounded pack-processing region. The block is handed to `FUN_180175c80`, but no final codec/bitstream read or decoded-audio effect is established. The six-value mapping and two packed slots are therefore a static input contract, not proof of DRC audio or BSI behavior. Runtime mode selection, selected-job serializer activation, and all six values' product approval remain unbound.

## Provenance

- Upstream `adm_bwf_ddp_drc_profile_input_boundary_wash_v1`, evidence manifest SHA-256 `05a1f8d1c7c7d0cb28c564df676954bbb83b440bbc937b31afdeb4b1fc06587d`.
- Wash independently matched all five payload files and all eight source evidence files; seal→manifest link matched. Upstream normal/`-O` checker results are reported PASS; Wash did not rerun them.
- Ghidra 12.2 static only; no DEE run or PCM/payload/key read. Raw code, decompilation, and scripts are excluded.
