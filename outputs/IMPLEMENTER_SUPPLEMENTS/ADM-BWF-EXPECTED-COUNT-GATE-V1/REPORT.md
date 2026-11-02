# ADM BWF expected-count gate and producer boundary

Recipient: implementation evidence supplement  
Disposition: an explicit expected-count field is present; its typed source is known locally but its factory-instance writer is not.

## Bound local count path

In `dee_audio_filter_convert_atmos_mezz.dll` (SHA-256 `9972be4f952ca5f9121fa83ef941b29d4c1e6baf0ff1f708baaaedffa1bd76a8`), selected `AtmosMasterFileStorage::Factory` methods pass the U64 qword at factory instance `+0x8` through `FUN_180244b90` / `FUN_180216360` / `FUN_180277700` or `FUN_1802798e0` into `FUN_1802769a0`. That constructor stores it in `AtmosMasterFileStorageDesc+0x6368` as the expected count.

The selected load path separately reads the reader's fmt-channel count from a qword at reader/storage `+0x18`, and checks it equals the expected `+0x6368` value. CHNA declared track count is a separate U16 validation against the same fmt count. A guarded branch (`owner+0x63b0 != 0`) rejects counts `>=129`; no fixed 36/38 comparison was found in the checked functions.

## Important unbound edge

The Factory instance `+0x8` field writer is not located. Thus its value cannot yet be classified as profile override, caller-preparsed fmt count, or caller-preparsed CHNA count. The equality gates alone do not prove that the authorized 36-track file reaches this factory with expected count 36. The 36/38 values are below the conditional 128 ceiling, but actual admission remains conditional on the expected-count and CHNA equality gates.

No DEE run, PCM/data-chunk read, encoded-payload read, or key/license access occurred. This package supplies no evidence of JOC algorithm parity and does not authorize padding/truncation.

## Upstream seal

- Package: `adm_bwf_ddp_expected_count_gate_wash_v1`
- evidence_manifest SHA-256: `0cff903a0014abcb572ed6fa405de23d2357df9826f15dbe1b4863e9c18a8869`
- package_seal.json SHA-256: `9fba18bb45b68b18df1eaaf6b4dee4fedb5a934fd8b87aa4e0d790ec98664778`
- Five payload sizes and hashes independently matched the seal; upstream normal and `python -O` checks passed.
