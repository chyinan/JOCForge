# Dialogue Intelligence Bool to measurement result dependency boundary

## Bound result calculation

The sealed measurement engine `FUN_1801769a0` receives the setup structure as `param_3` and initializes output buffer `param_2[5]` to `-120.0`. It iterates per-record readers and updates `param_2[5]` from returned measurement values at `lVar7+0x14`. The wrapper returns the same result buffer, and `FUN_18014aba0` copies `puVar12+0x14` to `Priv+0xf6c`.

## First break

The Bool is statically bound into setup `local_198+0x94`, and the setup structure reaches the engine. The bounded engine/first-consumer field check found no direct `+0x94` read; the result calculation is driven by per-record measurement values. No typed/field-specific edge joins setup `+0x94` to `param_2[5]`/`puVar12+0x14`. The Bool's causal effect on final measurement remains `UNBOUND`; runtime key selection remains `UNBOUND_NOT_CAPTURED`.

This is not a global no-effect claim. No DEE/UI/runtime, media/PCM/payload/key/license access, or implementation-source inspection occurred.

## Provenance

Source package `di_flag_bool_measurement_result_calc_boundary_addendum_wash_v1` manifest SHA-256 `91d0dd7dc34e5cebb3a58082df7c22cb33bc526ca0f2dce6231322db72c30c60`, package-seal SHA-256 `e6448d528f385a709860bb4dff10d1eb2cfdb2b39f9d87ad5be8f235f0ef5bed`; normal and `python -O` checks passed.
