# Selected temporal detector and weights

SYNTHETIC-QMF-FIT-CHECKPOINTS-V1 has manifest SHA-256 `7e07a290572784e3260899c5cbec14ce578a886ba51c8cdba6c0b09ed9177bc4`; all six distributed payload bindings match. INPUT-ONLY.json and approved providers are test inputs. EXPECTED.json is compiled only into tests and is read after the production function computes its proposal.

`TemporalWeightState` owns segment, 15x16 energy history, 15 last-change values, first-call state and the last 48 weights. Cold construction is `{changed=0,count=1,start=23,end=0}` with positive-zero arrays and `first=true`. `stage_target_windows48` consumes only current slots24..47 from object-major 15x48x64 complex-F32 target windows.

Energy, threshold, average, detection and history updates use the approved operation order with independently rounded binary32 multiplication and addition. The lazy energy cache is part of execution behavior: the three synthetic generations compute 241, 360 and 360 row/slot energies. Weights use the 1/24 recurrence for 23 up values, force the final up value to one, reverse the first 23 values for down and apply the before/new segment masks.

Local tests regenerate three current-QMF generations from the input recipe, then compare exact temporal before/after state, every energy-history word, last-change value, threshold and all 48 weights. Expected detections are slot0, slot12 and none. This proves the selected temporal substage on synthetic QMF values. Geometry, coefficient fitting, score, representation and production J05 admission remain separate work.
