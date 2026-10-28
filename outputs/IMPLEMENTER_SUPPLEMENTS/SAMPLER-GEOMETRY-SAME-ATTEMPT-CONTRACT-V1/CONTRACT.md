# Sampler and geometry source staging contract

Scope: selected 48 kHz, 38-physical-row, P11, regular-mode metadata composition. This is a Wash-only executable LOCAL_MODEL reference derived from approved F01E, F01B, F01GL, F03R and GTC contracts. It is not a new production API, a native identity claim, or a product acceptance result.

## Admission and owner identity

The caller first admits the input, job configuration, ADM graph, events and DBMD using the approved F01E rules. Bed and object rows must bind by physical host and channel identity, cover 0–37 exactly once, contain one semantic LFE, and contain 28 admitted objects. Each object event must satisfy the approved selected single-event, zero-start and complete-programme coverage constraints. The reference accepts already admitted event inputs; it is not an ADM, CHNA, DBMD or job parser.

Before each callback, bind the staged input, frontend state, source installation and parent transaction to one authoritative `programme_id`, `attempt`, input/config identity, callback index and transaction ID. A content hash verifies bytes but is not an owner capability. The reference compares fixture values and a fixture parent binding only; production must obtain the authoritative parent capability and same-attempt identities from its actual GTC plan and owner state.

## Normal callback and source installation

Let `N` be the admitted input's sample count and `A=ceil(N/1536)`. For `0 <= g < A`, the model input range starts at `1536*g`, with `min(1536,N-1536*g)` valid samples. F01/F02 owns input zero extension; this reference carries metadata only.

For each admitted object, normalize and clamp coordinates as in F01E.8. The cold first valid event initializes that object's previous XYZ to its current event XYZ. Then perform the specified two sampler slot updates, rounding each multiplication and addition to binary32 separately. Recompute control52 from the resulting per-object state, including the approved width and diffuse rules. The same event may remain constant across callbacks; the sampler state is still advanced on every normal callback.

Construct P11 sources 0–10 from the approved selected geometry rule. Traverse physical control rows 0–37 in order, skip only the one bed row whose semantic bed ID is direct LFE, and compact the other 37 rows into sources 11–47. Install each bed point by its already validated semantic bed ID. Install each object's current callback control52 and sampler XYZ in the source selected by that callback's physical row route. Prefix/bed attributes use the approved packed fields; object attributes use the approved bit packing from current control fields. Defined source fields are written only at the approved offsets. Source offset 0 hold/state is left outside this reference; no complete native structure is fabricated.

All source installation values are staged with the same parent callback generation that produced the controls. For this selected main clock, `g=0,1` has no codec source; for `g>=2`, source index is `s=g-2`. This contract does not implement audio activity, prefix writer sets, complete F03/F04 DSP, codec frames or publication.

## EOF, restart and parent receipt

After the last normal callback, four cached drains reuse its complete control snapshot; they do not advance the ADM event time or resample the event. A full restart discards event cache, sampler previous state, DBMD/physical binding, cached controls and counters and creates a cold owner for the next attempt. No prior-attempt metadata may be accepted as a new-attempt owner merely because its bytes match.

Preparation returns a complete staged after-state and proposed source projection without mutating the committed parent. The real parent must validate all downstream gates, then atomically commit under the approved GTC rules. Its existing owner receipts must carry the approved owner ID, before/after generation and canonical state hash, changed flag, commit count and transaction ID; the geometry/source owner is chosen by the approved owner projection, not inferred from an input hash. All contributing receipts must be bound to the same actual programme, attempt, transaction and callback generation.

The executable reference emits `before_trace_sha256`/`after_trace_sha256` over a factored local test state. They are diagnostic model digests, **not** approved canonical GTC owner hashes. The reference holds admitted metadata outside its test state and does not serialize the approved nested event cache, physical binding, DBMD binding or complete owner root. It creates no production owner receipt or parent commit.

## Validation scope

Input vectors include one primary ADM metadata fixture and a separate approved synthetic clock fixture. The model is evaluated over 4,247 normal callbacks and four cached drains for that fixture; selected checkpoints carry complete controls and source projections. Additional synthetic checks cover interleaved LFE placement, stale-attempt and transaction/generation mismatch rejection, warm sampler dependence on previous state, and two different synthetic input lengths. These are model checks. No real audio, native execution, E-AC-3 JOC encode, player test, seven-file validation or product acceptance is claimed.
