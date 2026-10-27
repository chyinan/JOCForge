# F07-F12 selected owner amendment: transform state and codec root

This amendment reconciles the conflicting owner fields only. It preserves the other owner schemas and all F08/F12 numerical rules.

## Authoritative owner split

The six recovered 128-word rows are primary F08 transform state. The rev10 `AnalysisState.restored_previous[6][128]` is read by the primary long/short/LFE transform calls. Map its external rows `[L,R,C,LFE,Ls,Rs]` into the already approved Analysis owner primary bank: FBW internal order `[L,C,R,Ls,Rs]` uses external rows `[0,2,1,4,5]`, and the separate primary LFE row uses external row `3`. The Analysis owner already contains these primary histories and the separate companion FBW/LFE histories. Do not create a duplicate six-row Codec owner.

The selected persistent F12 codec state contains its existing U64 generation and the five `previous_short` I1 values in codec order `[L,C,R,Ls,Rs]`. Preserve the existing canonical JSON field paths where possible: codec generation and `previous_short.i1_words[5]` remain in the same F07_F12 owner projection and are admitted by one GTC owner receipt. The source index remains owned by the existing GTC source clock; do not add a duplicate `next_source` field.

## Superseded projection fields

For the selected clean production root, supersede only these fields in `F07_F12_OWNER_STATE_V5`:

- `codec.transform_history_u32_words=256`: this is the fixture audio-owner projection of external L `delayed_audio[0][1280:1536]`, not a persistent codec transform history. The full primary transform histories belong to Analysis/F08.
- `codec.mantissa_group_state_u16_words=18` and `codec.bitwriter_cursor`: these are not persistent state. Groupers reset per block; candidate trials get fresh group and bitwriter scratch.
- The statement that the six primary transform rows are an additional F12 Codec owner is withdrawn.

No other history, provider, profile, error, or transaction rule changes. The old `CODEC-FIXTURE-SEAM-V1` remains a test-only fixture projection and has no production owner authority.

## Transition and canonical evidence

F08 advances primary and companion histories once on the staged Analysis root for the current source. F12 computes staged `previous_short` from current six-block short flags and old flags before candidate trials. Candidate failures preserve both staged owner proposals for later candidates; all-candidate failure discards them and leaves committed bytes unchanged. An accepted suppressed source 0/1 still requires a full 4096-byte frame and CRC before parent admission, but appends no AU. A published source requires the existing F15 durable append receipt before one parent root swap admits Analysis, codec, and other owners together. No codec-private EOF flush is added.

Reuse the approved F07_F12 owner canonical JSON rule and GTC owner before/after identity. Do not introduce the proposed binary magic/profile/provider codec header, a second state digest, or a separate leaf commit. Cold codec generation is zero and previous_short is `[0,0,0,0,0]`; full reset reconstructs the existing Analysis cold state and these codec values together.

## Evidence grade and limits

The approved F08 and source composition contracts define the primary/companion histories, their cold values, and one F08 pass. Hash-matched restored rev10 source and long/short/LFE helpers show the six-row primary dataflow; they are an earlier model revision, not selected rev13 byte identity. Selected rev13 source/executable caller identity and native owner join remain `NOT_RUN`. This local owner reconciliation is sufficient to avoid duplicate mutable history in the clean implementation; it does not claim native ABI or full-chain execution.
