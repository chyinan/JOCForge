# GTC METERING/READY owner availability: local schema amendment

Status: Wash-approved LOCAL_MODEL amendment for the selected clean implementation. The approved v2 ProgrammeState JSON Schema requires twelve initialized ownerState objects at every status, while GTC MeasurementTransaction constructs fresh main roots only after METERING→READY. Some selected owners depend on the finalized receipt, and the inspected approved materials do not specify real pre-measurement refs for all eleven non-meter owners. Do not invent hashes to satisfy v2.

## Versioned state representation

Introduce a new local ProgrammeState schema version (v3); keep v2 schemas, hashes and fixtures immutable. Preserve all twelve owner IDs in the map. Each value is a tagged union:
- UNINITIALIZED: explicit discriminator only; no fabricated generation/hash/state_ref.
- INITIALIZED: the current exact owner_id, generation, state_sha256, cold_sha256 and state_ref fields.

During METERING, meter is INITIALIZED from the actual aggregate meter root; the eleven main-path owners (reader_eof, frontend_control, spatial_control, spatial_audio, joc, drc, analysis, codec, metadata, integrity, output) are UNINITIALIZED. Meter callbacks commit meter only. They do not seed main state.

After successful meter finalization, one parent MEASUREMENT commit validates the terminal meter root, immutable MeasurementReceipt, attempt/input/config identity, actual schedule and getters. In the same atomic METERING→READY transition, retain the actual terminal meter ref and install the eleven main-owner cold states from their respective approved owner constructors/providers, bound to that programme/attempt/prepared identity. DRC receives the receipt-derived dialnorm; metadata gets its approved cold/timer state. Each owner must supply its actual canonical bytes, cold identity, generation and state_ref. No owner becomes visible until the full set validates.

If finalization or any owner construction fails, install no partial main roots and do not issue READY or MeasurementReceipt; apply the GTC fatal/RETIRED rule when the failure is fatal. Restart increments attempt, discards receipt and owner availability, and creates a fresh METERING root with a new meter cold root and eleven UNINITIALIZED main slots.

## Generations, receipts and serialization

phase_generation advances once per successful parent commit. Meter generation advances on successful meter callbacks. Newly initialized main owners use generations required by their own cold constructors (normally zero). Receipts bind owner, before/after generation and hashes, changed, commit_count and transaction ID. UNINITIALIZED→INITIALIZED must have an explicit construction receipt with before availability; never fabricate a before hash.

The v3 root needs a new canonical schema identity. Do not reinterpret v2 bytes or KATs. Reuse each owner’s approved serialization. This is a local state-model change, not a claim about DEE transaction internals.

## Input-only acceptance cases

Add red-first cases for: METERING with only meter initialized; meter callback changes only meter; successful finalize installs all eleven cold owners plus receipt atomically; wrong attempt/input/config or missing provider rejects with no partial install; fatal finalize retires without receipt/READY; restart creates a new METERING attempt with fresh meter cold state and eleven uninitialized main slots. Derive expected hashes from actual constructors at test execution; fixtures never seed production state.

Authority: approved GTC spec SHA 5415b7b2fb60f50351cd26691e7ef0fc5bd758e501d305d0a714b90c714cd105; ProgrammeState schema SHA 74c8e4a60600187783e559459b426aada2508ae273f95401c9a4234aee4504b4; F06 finalize package manifest abb560381448727aa4c88877373769aa14556a4e365e4bec82637a043155333b; Metadata owner package manifest 94774c412ce8457f3220d84e4b3386c0b48c21837d4a4943570791bb66f034b3. This amendment resolves status-dependent availability absent from v2. Production implementation and real READY execution remain NOT_RUN.
