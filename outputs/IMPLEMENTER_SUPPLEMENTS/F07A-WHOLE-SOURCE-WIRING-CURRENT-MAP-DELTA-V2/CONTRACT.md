# F07/F07A current map and implementation boundary

Status: `CURRENT_MAPS_RECHECKED; APPROVED_RULES_PRESENT; WHOLE_SOURCE_AND_PARENT_GAPS_OPEN`

The approved package already contains the F07/F07A rules, provider identity, and input-only recipes. This supplement refreshes the mapping against the current implementation checkout; it adds no new algorithm formula and does not claim that the implementation or native encoder ran.

## Approved source-frame rules

For one source frame, F07A transient analysis consumes `installed_audio` before the F07.1 delay; F08 consumes that same frame's `delayed_audio`. External channel order is `[L, R, C, LFE, Ls, Rs]`. FBW lane order is `[L, C, R, Ls, Rs]`, mapped to external rows `[0, 2, 1, 4, 5]`; LFE is external row 3. Each lane consumes blocks 0 through 5, with 256 samples per block, giving 30 FBW lane/block calls per source frame. LFE makes zero F07A transient calls and follows the separate F08.7–F08.9 seven-bin primary/companion path.

These rules and their source references are already in the approved package. The current local 36-block D05 slice is one FBW lane and does not establish a five-lane source-frame orchestrator. The 33-word D05 provider slice parses provider identity only; it does not filter audio or update state.

## Current mapped gaps

1. No five-FBW source-frame orchestrator is mapped. Implementing the approved source-frame composition remains distinct from the one-lane D05 leaf.
2. The selected callback's owner/source-generation join and the parent staged success/fatal/reset/EOF commit edge remain unbound. Parent-stage evidence is not a production execution receipt.
3. No current map binds all five FBW outputs and next-state values from F07.4/.5 to the production parent consumer and commit edge.

The F07A provider remains a provider-only slice. F08.2 fold/history is an adjacent single-block leaf, not whole-F08 or parent-source integration. These boundaries must stay explicit in the implementation map.

## Implementation use

Use the hash-bound F07 specification, D05 provider/authorization records, and approved input-only vectors listed in `SOURCE-MANIFESTS.json`. The pure source-frame orchestration can be developed against those contracts while the production owner/generation and parent commit integration remains explicitly staged and unbound. Do not infer whole-programme completion from synthetic vectors or local leaves. Do not start further static reverse searches for formulas already present in the approved contract; request a targeted wash only if implementation exposes a concrete missing typed edge.

The research package also records a discarded malformed text-search incident. Its search output was discarded and not used; binary media was not intentionally scanned, but the package cannot rule out text-encoded samples from the discarded text search. No sensitive output was retained. Preserve this disclosure as evidence scope, not as an algorithm result.
