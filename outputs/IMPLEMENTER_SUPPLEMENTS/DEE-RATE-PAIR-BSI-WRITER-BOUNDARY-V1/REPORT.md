# Two-value data-rate path to codec writer boundary

## Bound static values

On the inspected inherited mode-two route, `384` maps to selector code `0`, while `1024` maps to selector code `5` if the lookup is reached. `FUN_1801ac130` writes typed setup value `6` for code `0` and `8` for codes `1` through `5`; `FUN_1801b2250` builds the typed codec-configuration candidate.

The same inspected process path reaches `FUN_1801b1590` through `FUN_1801ab800`, loading a writer-state pointer from `param_1+0x810`. Its per-stream writer call is a dynamic dispatch through table `DAT_1806521e8`, slot `+0x28`.

## First break

The rate-derived configuration candidate at `piVar17+0x206` is not statically joined to the writer-state pointer at `param_1+0x810`. The selected dynamic writer target and the BSI bitrate field remain `UNBOUND`; the package does not establish that setup values `6` or `8` are BSI fields.

This is evidence for two requested values only, not all supported bitrates. Runtime selection, codec output and audio effect remain `NOT_RUN`.

## Provenance and scope

Source package `DEE-ADM-ATMOSMEZZ-RATE-PAIR-BSI-WRITER-WASH-V1` was checked in normal mode and under `python -O`. Ghidra 12.2 static evidence only; no DEE execution, media/PCM/payload/key/license access, implementation access, or process control.
