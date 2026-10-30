# Selected ID1 and ID2 body validation

Scope: clean selected model and programme-bound source transaction. F13.6–.8 (`specs/13-emdf-and-timeline.md`), F13JO.2–.3 (`specs/13-joc-complexity-timer-payload-order.md`) and F13D2.1–.4 (`specs/13-id2-profile-metadata.md`) define the body producers and wire fields. A validator can consume and check all fields without retaining a public decoded object; it must still consume the entire bounded body, check all conditional/reserved fields, enforce selected-value constraints and bind to its producer. A header-only/length-only check does not constitute body validation. Semantic checks do not require a new decoder/rendering pipeline.

## ID1 grammar

Read from the exact bounded body MSB-first. Require version U2=0. Read field0-present U1; if 1 read field0 U3 (0..7), else field0 is absent. Read field1 U4; if nonzero read field2 U1 and field3 U1, otherwise those two fields are absent. Read fields4..8 in order: each starts with U1 presence; if present read an unsigned code of the indicated width and add the indicated minimum:

| field | width | min | max | permitted code |
|---|---:|---:|---:|---:|
| 4 | 7 | -116 | 11 | 0..127 |
| 5 | 7 | -116 | 11 | 0..127 |
| 6 | 8 | -232 | 23 | 0..255 |
| 7 | 8 | -232 | 23 | 0..255 |
| 8 | 5 | -15 | 15 | 0..30; reject 31 |

Absent fields4..8 map to sentinel -512 for model comparison. Read final flag U1=0. Then allow at most seven remaining bits and require each to be zero; a whole extra byte is not padding. Reject truncation at any conditional field, unsupported version/final flag, out-of-range field8 code, nonzero/excess padding, wrong selected provenance or body >10 bytes. The local validator need not guess a canonical representation of absent field0 or absent field2/3: retain presence separately and compare the exact expected presence and fields.

F13.6 constructs base fields `[-2,1,int(preferred),0,half(ITU),half(dialogue),-512,half(true_peak),-512]` from a completed same-programme F06 measurement and explicit preferred bool. F13.7/F13JO.2 use timer parameter `[24000,48000,48000,4800,24000,48000]`, the bound before timer state, the current writer's source call and 1536-sample hop to generate the actual output fields and staged after state; only one expired field may refresh per call in priority order. The body encodes that timer output, not directly the unmodified base config. The encoder sets field presence based on `value!=-512`; a `-1024` timer output is present and clamps to the lower bound, not absent. A parser observing the lower-bound code cannot recover whether the producer used that bound or a lower input; validate against independently produced staged timer output by recomputing its normative presence/clamp projection. Do not reverse infer the original loudness or timer input from body bytes.

For selected base input field1=1, field2 is the explicit preferred value and field3=0. The full producer result and actual state progression remain authoritative for field0 and fields4..8. Do not replace same-writer timer output with hard-coded default fields, a prior source or fixture bytes.

## ID2 grammar

Require exactly 3 bytes. In bit positions 0..20, MSB first:

| position | width | selected rule |
|---|---:|---|
| 0..1 | 2 | version/mode 0 |
| 2..4 | 3 | optional presence A/B/C all 0 |
| 5 | 1 | phase block present 1 |
| 6 | 1 | phase reserved 0 |
| 7 | 1 | phase bit 1 for selected input |
| 8 | 1 | LFE bit from same-writer ID11 header |
| 9..10 | 2 | reserved 0 |
| 11 | 1 | DRC profile present 1 |
| 12..14 | 3 | line code 0 for selected none |
| 15..17 | 3 | RF code 0 for selected none |
| 18..20 | 3 | trailing optional presence A/B/C all 0 |
| 21..23 | 3 | padding all 0 |

F13D2 primitive also maps other exact external profile strings to codes, but that mapping does not widen the selected none/none programme. Derive LFE from the same-writer owned ID11 body: read version U2=0; count_code U5, where count=count_code+1 if <31 or 32+following U7 otherwise; shortcut must1; read LFE U1 and require count>=LFE. This is a local header read; full ID11/OAMD validation is separate. With selected phase_indicator=1, format_mask=1, lfe_mask=LFE, phase bit=(indicator!=0), LFE bit=((format_mask & lfe_mask)!=0), and both DRC codes0. Check actual body against these current producer-derived bits. Do not use a different writer's ID11 or an old ID2 body.

## Queue and same-container relation

At SourceTransaction w, the independently successful ID11, ID1 and ID2 body/proposal inputs all bind the current programme/attempt and writer w; the ID1 timer receipt references the actual staged timer before/after state and completed same-programme measurement, while ID2 references the same-writer ID11 body/receipt and typed job profiles. Actual owned bytes plus provenance and receipt/transaction identity are required; matching values or hashes alone are not a join. F13 owns the queues and stages push/pop order [11,1,2,14]. For publication at source s>=2, the ID1, ID2 and ID11 popped bodies each have original writer w=s-2; validate the ID2 LFE relation against that popped ID11, not current push writer s. Keep original producer transaction separate from current consumer transaction and bind each popped body to actual bytes in the current protected EMDF. ID14 comes from writer s-1. Source0/1 have no published EMDF and skip this parser; they still execute their producer and queue stages.

At producer preflight validate ID1/ID2 ≤10/8 FIFO bytes; selected ID2 semantic grammar narrows actual length to3. F13 wrapper configuration is separate (ID1 duration1536; ID2 discard=true) and does not change the body syntax. Reject producer mismatch, malformed body, unsupported selected code, wrong source/attempt/queue binding or invalid complete container before committing staged owner state. GTC parent governs failure and retirement; no unproven native rollback is claimed.

## Evidence

The attached input-only cases describe source fields and mutations, not expected bytes. A small independent synthetic model serialized and consumed three ID1 cases (including a present -1024 that clamps to -116) and selected ID2 LFE0/1, checked truncation/trailing bytes, and passed regular Python and `-O`. It is a same-rule roundtrip, not native, complete timer execution, current protected-container integration or real ADM evidence. Approved timer input vectors exercise timer schedule and capacity separately; local producer/parser/full-song execution and spatial playback remain NOT_RUN.
