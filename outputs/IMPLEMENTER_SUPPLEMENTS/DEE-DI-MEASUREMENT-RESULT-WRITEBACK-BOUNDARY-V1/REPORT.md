# Dialogue Intelligence measurement result writeback boundary

## Bound result path

In the sealed `Priv::measure` route, `FUN_18014aba0` calls wrapper `FUN_180176980` with the result buffer pointer and the measurement setup structure. The wrapper returns the result-buffer pointer; the caller reads the U32 field at `puVar12+0x14` and writes it to `Priv+0xf6c`.

## First break

The previous Bool receipt binds `dialogue_intelligence`'s static Bool candidate to setup field `local_198+0x94`. The current result receipt binds a separate result-buffer field `puVar12+0x14` to the final `Priv+0xf6c` writeback. No typed/pointer lineage joins setup `+0x94` to result `+0x14`; therefore the Bool's effect on the measurement result remains `UNBOUND`. Runtime selection of the query key is also unobserved.

No DEE/UI runtime, PCM/media/payload/key/license access, or implementation-source inspection occurred.

## Provenance

Source package `di_flag_bool_measurement_result_writeback_addendum_wash_v1` manifest SHA-256 `52b9d7038edf33c0012401ada04e3add9e4da34c014e28b1a62fed9254b214ae`, package-seal SHA-256 `ca1bf3f9624c05f6a6f0d542431654e155d421798aa823cc96fff8c2a96c58dd`; normal and `python -O` checker passed.
