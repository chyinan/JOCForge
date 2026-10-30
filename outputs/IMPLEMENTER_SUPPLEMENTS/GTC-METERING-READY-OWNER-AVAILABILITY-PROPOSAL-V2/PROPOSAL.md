# GTC METERING to READY owner availability proposal

Status: PROPOSAL_PENDING_USER_APPROVAL. This is a local schema amendment proposal only. It does not change the approved v2 schema, baseline, or implementation authority.

## Conflict
The approved v2 schema requires all twelve initialized owner references in every status. GTC MeasurementTransaction runs meter independently and constructs fresh main roots only after successful measurement. The reviewed sources do not define pre-measurement owner bytes for all main owners; DRC/ID1 depend on the completed receipt. Placeholder hashes would falsely describe state.

## V3 schema
The attached complete v3 JSON Schema preserves every v2 field, ledger, receipt, invariant and owner ID. It changes owner slots to an explicit tagged union:
- UNINITIALIZED: owner_id and availability only, with no fake hash/ref/generation.
- INITIALIZED: owner_id, availability, generation, state/cold hashes and state_ref.

METERING requires meter INITIALIZED, measurement_receipt null, and eleven main owners UNINITIALIZED. READY/ACTIVE/FINALIZING/DONE require a MeasurementReceipt and all twelve owners INITIALIZED. RETIRED availability is transition-dependent and must equal the last committed snapshot; the transaction validator enforces this.

## MEASUREMENT parent transition
Before: committed METERING root with actual meter state/ref, receipt null and eleven uninitialized slots.
Prepare: validate programme/attempt/input/config identity, exact meter schedule, terminal canonical meter bytes and getters; construct immutable MeasurementReceipt; build each main cold state through its owning approved constructor/provider, with DRC dialnorm derived from the receipt. No fixture or placeholder state is allowed.
After: one parent swap retains terminal meter ref, installs receipt and all eleven actual cold refs, sets READY and increments phase_generation exactly once. Eleven construction receipts state before_availability=UNINITIALIZED and actual after identities. If any preparation fails, no partial installation or READY; fatal handling follows the approved GTC rule.
Restart: increment attempt, discard receipt, create fresh meter cold state and eleven UNINITIALIZED slots.

Owner-specific serialization and constructors remain with approved owner schemas. This proposal changes the root schema version and requires a new canonical v3 identity; v2 KAT bytes remain historical and unchanged.

## Input-only transition cases
1. Metering cold root: meter initialized, receipt absent, eleven slots uninitialized.
2. Each meter callback changes only meter after-state and monotonic clock.
3. Successful finalize installs exactly eleven valid owner cold states and receipt in one parent commit.
4. Missing owner provider, stale attempt, or mismatched input/config rejects with committed root unchanged.
5. Failure halfway through cold construction installs nothing and exposes no receipt/READY.
6. Fatal meter stage retires without a measurement receipt.
7. Full restart creates a new attempt with fresh meter cold state and eleven uninitialized owners.

All expected hashes must be computed from actual constructors during execution; no fixture hashes seed production.

Sources: GTC spec 5415b7b2fb60f50351cd26691e7ef0fc5bd758e501d305d0a714b90c714cd105; v2 ProgrammeState schema 74c8e4a60600187783e559459b426aada2508ae273f95401c9a4234aee4504b4; F06 finalize supplement abb560381448727aa4c88877373769aa14556a4e365e4bec82637a043155333b; metadata owner supplement 94774c412ce8457f3220d84e4b3386c0b48c21837d4a4943570791bb66f034b3.

The v2 conflict itself is confirmed; this proposal selects one resolution for user review. No production READY execution is claimed.
