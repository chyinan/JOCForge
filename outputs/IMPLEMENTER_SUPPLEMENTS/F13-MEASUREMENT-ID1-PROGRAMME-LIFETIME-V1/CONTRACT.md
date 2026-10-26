# Final measurement and ID1 programme lifetime

This supplements the ready model field crosswalk in F13-PREFERRED-F06-MODEL-FIELD-CROSSWALK-V1. It specifies the model's programme lifetime; native factory/configuration lifetime remains a separate investigation.

The final selected measurement result checks that its owned energy and ITU inputs belong to the same programme and have the required complete-window cardinality. Dialogue selection, ITU and peak scalars are separate model-owned results with explicit input identities. They are not a raw native 44-byte record. Use this finalized measurement set to build the nine-integer ID1 configuration once for the programme, including preferred from the public result kind. Timer updates reuse that configuration for every source; measurement selection is not repeated on each source.

The duration core constructs fresh timer state and fresh metadata queues. Its loop count is the supplied metadata/JOC frame count, checked against the other frame domains. A retained old 96-call or diagnostic label does not limit this loop. At EOF it reports remaining queue state and does not remeasure, change the public result kind, or derive a new preferred value. A model restart constructs cold state again. Pending payload publication/discard rules remain those of the approved F13 timeline contract.

Keep the approved full F06.2/F06U selector, including fresh ungated fallback when ITU is unavailable. The old frozen helper's valid-ITU-only restriction is historical scope, not a product rejection rule. The included scalar recipes test selection/domain boundaries only; they do not compute a programme's actual energy, ungated mean or complete measurement.

Enum domains remain distinct: internal ungated/level/dialogue = 0/1/2; public result = 0/2/1; wrapper = 1/3/2. ID1 preferred uses public kind == 1. The public result field can be carried by the implementation's existing final measurement type; DEE need not have a Rust receipt, digest or global transaction with that name.

The source investigation used exact frozen code and archived stage/scalar identities. Wash rehashed the package, all nine source/contract files, three scalar metadata documents and the timeline ledger, and independently replayed six scalar boundary cases from the approved selection law. No source code, archived measurement metrics, raw native records, body bytes or PCM are forwarded. These checks are not a new native or whole-programme execution, and do not prove native member identity or selected factory values.
