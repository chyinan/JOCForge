# Direct preconditions for real callback0 admission

## control38

Known facts:

- owner is frontend_control;
- value is callback-local U32[38][13], 494 words, row-major host order;
- it is derived only after physical binding preflight;
- ten bed hosts and twenty-eight object hosts are disjoint and cover 0..37;
- the unique semantic LFE host may occupy any physical host;
- object rows consume the corresponding object ordinal event/sampler and DBMD
  track; bed rows consume semantic bed/DBMD rules;
- regular37 is a view excluding the resolved LFE host and is not independent
  sampler state.

The physical callback0 material is captured in
CONTROL38-CALLBACK0-MATERIAL.json. It contains all 494 U32 values and the
38-entry physical binding. DBMD/event binding is represented by the SHA-256 of
28 derived 88-byte runtime records; those record bytes are not delivered.

The producer is dee_audio_filter_ddp_atmos.dll RVA 0x1ACD10, called from RVA
0x1A780D under parent RVA 0x1A7710. It reported 38 rows with 52-byte stride and
returned success. The observer was read-only. The extracted 4247-frame EC3
remained byte-identical to the accepted baseline. No old fixture hash or
default host mapping was used.

Input lineage is bound by CONTROL38-INPUT-LINEAGE.json. The DEE -a input was a
native-safe alias with the same 743,696,526-byte length and SHA-256 as the
pinned REAL-ADM identity. The original fixture identity and alias identity were
both checked before and after capture, and the run record reports stable
assets. Paths and raw input bytes are not included.

The DBMD track-to-bed/object semantic binding map is now delivered. Native
mapping codes, CHNA/AXML track and object identity, parsed DBMD track fields,
event/sampler ordinals and control-row identities are joined per physical row.
The derived record hash alone was not treated as proof; closure requires the
combined mapping and 28 object control rows matching word for word.

Semantic mapping capture is distinct from callback semantic admission and
callback numerical admission. Both admission steps remain NOT_RUN. Remaining
evidence is a frontend_control callback0 after-state/snapshot commit identity
and a callback-level semantic admission receipt/owner token. The row mapping
itself is not blocked.

## Authoritative parent cold state

Known constructors:

- F01E cold nested JSON is 4402 bytes with SHA-256
  9c943879cd36cb45bc7538346cfdf4f71d0998d8f261f75c135045fcb59870aa;
- callback/source clocks are zero, remaining cached drains is four;
- source caches and snapshots are absent and last phase is UNSET/0;
- render numeric histories are positive zero and diffusion instances absent;
- feature source vector/scalar are positive zero;
- selected target cold values are pinned by their provider;
- assignment is -1, markers/matrix are zero and writer-set is empty.

Authoritative complete bytes remain BLOCKED. Missing evidence:

1. exact top-level envelope/version and section framing;
2. exact embedding/framing of the nested F01E canonical JSON;
3. exact optional diffusion/source-cache record framing;
4. whether current_flags and current_features, described as callback outputs
   with no independent history, are serialized in persistent cold state;
5. exact matrix writer-set record framing, despite the cold count being zero;
6. one authoritative serializer execution proving section offsets and total
   length.

The state schema's descriptive field order does not resolve those byte-level
questions. AI48ST01 test bytes are explicitly non-production and cannot be
used. The minimum evidence is one cold-constructor serialization with section
offsets, lengths, complete preimage and SHA-256, plus an independent parser
round trip that proves no scratch or unknown zero-fill is present.

Neither blocker requires spatial render, selector, U04 or outer execution.
