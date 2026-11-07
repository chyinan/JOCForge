# AtmosMezz data_rate to native selector boundary

## Selected job and schema

The hash-bound selected job is one `audio.atmos_mezz` → `encode_to_atmos_ddp` route with XML text `data_rate=1024`; the input is the user-confirmed Human Cat ADM identity recorded in the source receipt. The XML and source identity hashes are recorded in the evidence package; the ADM hash was not recomputed in this pass. The schema declares `data_rate` as an `xs:list` of string tokens `[384, 448, 576, 640, 768, 1024]`, default `448`. Only 1024 appears in the selected job; the other values are schema alternatives, not run results.

## Native selector and first dimension split

`FUN_180178dc0` reads an already-converted U32 code `0..5`, writes it to `open_config[10]`, and errors for invalid codes. The scoped trace does not bind XML text `data_rate` to the source word. `FUN_1801535b0` has a separate workflowParams diagnostic read of the `data_rate` key; it does not assign the open-config selector.

The first observed selector-dependent consumer is `FUN_1801ac100`: code 0 yields dimension 11, while codes 1..5 yield 15. On the normal open branch the caller adds one, producing a 12- or 16-dimension path into `FUN_1801acfe0`/`FUN_1801ad2d0`. `FUN_1801abc10 → FUN_1801ac130` maps the same code to a further field value of 6 or 8. The earliest observed split is therefore native selector code 0 versus codes 1..5—not a known XML bitrate split.

## Exact open boundary

The six XML tokens share the same provider and static open/process route at the metadata level. The mapping from numeric XML tokens to native selector codes remains `UNBOUND`; no per-value branch can be assigned to 384, 448, 576, 640, 768, or 1024. The normal-open flag value is not captured, later writer behavior was not evaluated, and no DEE encode was run. Do not infer support of all rates from the schema list or from the selector's accepted code range.

## Provenance

Upstream `MANIFEST.sha256` file SHA-256: `2352b6835dd2d6b202dd71aa96ddaf6019487670b2d2c1609d70a20c6b03d191`. Wash rehashed all three listed package files, all twelve referenced analysis artifacts, the selected job XML, schema, and source evidence; hashes matched. Ghidra 12.2 read-only/no-analysis. No PCM/payload/key/license or implementation access occurred; the obsolete `sample6-keyless-full-20261101-08.ec3` artifact was not used.
