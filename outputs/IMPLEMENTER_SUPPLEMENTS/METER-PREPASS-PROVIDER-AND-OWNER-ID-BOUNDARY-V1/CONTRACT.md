# Meter prepass provider coverage and owner identity boundary

Status: ASSET_IDENTITIES_BOUND_BUT_TRANSITIVE_RUNTIME_COVERAGE_INCOMPLETE. This is a washed boundary contract. It does not authorize production prepass, terminal receipt, READY, or use of synthetic providers.

## Independently checked package references

- METER-DIRECT-PROVIDER-INVENTORY-V1 manifest 9295dc88c5fbd696ccb13e8c08527bae08748fecfc8d7fbe0f3c2b12ddb848e7: seven direct entries, including six numerical assets and one broad dependency index. Its contract says the direct inventory is non-exhaustive and provider loaders, semantic validators, and prepass were not run.
- METER-FRONTEND-PROVIDER-EXPANSION-V1 manifest aff35d4d426dc5ff203bda1bf55c936b0ffb054bb2d544c4f6bff35048c80c8f: 48 upstream index entries, covering specifications, parameters, and review/vector records. These are not 48 runtime providers.
- F01B-F03W-COLD-STATE-ASSETS-V1 manifest d15672b723f8091b576c2228ed7685744f522706d89078050b6486a72a068372: five exact selected cold/state assets, not warm state or measurement.
- GTC-MEASUREMENT-READY-APPROVED-CONTRACTS-V1 manifest 8a7410f0d5049d40196c3baf3f4a074d1693e11c65517440cb6f5368632a369e: approved 12-root transaction, measurement, clock, restart, and READY rules; execution remains NOT_RUN.
- F06-METER-INPUT-CURSOR-MODEL-OWNER-KEY-BOUNDARY-V2 manifest 3dc1750db282cc0cf1ff56b3fcc936f0458dcd53a5a967f9e04c864317f282cc: frozen Panner model key only; no selected Meter production ingress caller or Meter owner-key producer was found.

Wash rehashed each listed package manifest and members. The direct inventory is a coverage boundary, not a complete dependency set.

## Values already available

The approved parameters/window-q15.json hash is a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130. It includes 32 Q15 receiver_blend32 values; the approved accessor is ValidatedWindowQ15Provider.receiver_blend32(). The 64-word component_kernel64 is a separate parameter.

The seven direct meter inventory entries are: meter-truepeak-authorized-v1.json; meter-oar-secondary8-math-v1.json; meter-log-mathematical-v1.json; meter-k-state-bounds-v1.json; meter-k-matrix-math-v1.json; shared-dv32-mathematical-v1.json; and adm-installed48-providers-v1.json. The last item is an upstream dependency index, not loaded numerical values. Meter owner families include input/frontend/component, reader controls/LFE, OAR, DV32, K/window/energy, integrated/ungated, true peak, and terminal receipt. Upstream F01/F03/F04/F05 selected-input values are also required. The current package set does not establish an exhaustive validated transitive owner set, so a complete provider aggregate digest is NOT_AVAILABLE. Do not call a hash of the seven direct entries, 48-entry index, or cold fixtures a complete provider digest.

## Minimal local MeterSourceOwnerId rule

The approved GTC contract gives one programme attempt one root. attempt is zero-based; fatal failure retires the attempt; full restart begins at the programme input origin, increments attempt, and rebuilds cold owners and clocks. A native pointer token is not required for a safe local parent identity.

Minimal local application: the programme parent mints a typed MeterSourceOwnerId for the admitted source stream within that root attempt. A typed admission receipt binds it to programme_id, attempt, PreparedJobInput identity/config, selected profile, and validated source extent (sample rate, channels, frame count/clock). Normal callbacks and the three fresh-zero meter drains retain that identity. RETIRED/full restart invalidates it; attempt+1 receives a new root and owner ID. Do not derive the ID from a path, fixed label, native pointer, or input hash alone. This is an internal application of the approved transaction contract, not a DEE-native key or authentication credential.

## Remaining minimum work

Enumerate every consumed owner-specific value from selected F01/F03/F04/F05 manifests, bind it to an owner semantic validator and actual loaded immutable value, then create the provider receipt only from that complete validated set. Run Q normal callbacks plus three actual fresh-zero meter drains on an authorized admitted input, validate terminal state, and produce the MeasurementReceipt under the parent transaction. Fixture-driven ignored tests and METER_ROOT_V4_COLD do not count.
## R3 owner-by-owner coverage ledger

The included OWNER-PROVIDER-LEDGER.json is the hash-verified ledger from reverse agent 3. Its source package manifest is e287f082257ca42401aa485a7eaa0eecb832faa18a050d3c2b76d4a9b844138e; the ledger payload is 66822fc14197c7f9e02c14588a0d8e9e03c3bd3fb21c15acabf497c22d54d483. It records 48 upstream index entries, 7 direct meter assets, 5 cold assets, and 12 GTC roots. It classifies the upstream entries into 33 specifications, 11 parameters, and 4 review/vector items. All 12 roots have actual_prepass=NO and committed_runtime_owner=ABSENT. For the meter root specifically, ingress owner key, extent, cursor, and terminal binding are UNBOUND; direct assets are documented but loaders/validators are unexecuted. The ledger is coverage evidence, not a production-ready provider set.
