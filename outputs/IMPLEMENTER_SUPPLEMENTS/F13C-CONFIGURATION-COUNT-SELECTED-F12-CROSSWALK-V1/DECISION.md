# Configuration count to BSI and selected F12 crosswalk

Status: `CAPTURE_LOCAL_COUNT_PATH_CLOSED; SELECTED_MODEL_CONSUMER_EXPLICIT; NATIVE_RECORD_TO_SELECTED_F12_JOIN_NO`.

The evidence establishes a bounded configuration-count path into a BSI importer field and separately establishes the selected F12 model source for its JOC complexity field. It does not connect the captured configuration record to a selected source frame or AU. Keep that cross-path identity `NO/UNBOUND`.

## Bounded configuration-count path

In the reviewed ordinary, non-bypass wrapper capture, `rate_code=5` and 38 signed-I16 channel labels produce 15 non-LFE channels plus an LFE-presence flag of 1. The configured count is a signed I32; parameter 13 queried with selector 10 returns 16. Within that capture, the query result reaches a configuration-record constructor, the BSI producer consumes that record, and an importer consumes the produced record.

The imported record's identified destination is `complexity_index_type_a`, an 8-bit field at native frame bit 105 (cursor 113 afterward; working-header byte offset 119). The constructor input is the bounded freeze point for this path: the query value is placed into the configuration record before the BSI producer consumes it. The local capture reports seven producer calls and six importer calls, but no source/AU tags. The count contract establishes only the observed selected value 16 and bounded successful calls at counts 1 and 16; it does not define a general legal range.

## Approved selected F12 model path

For a published source with a complete validated same-source EMDF, the approved F12 wire contract emits an 8-bit JOC `complexity` value derived from that EMDF: ID14 `object_count=15`, ID11 `slot_count=16`, and `complexity = U8(object_count + 1)`. This is a model-side input to the additional-BSI writer. Sources 0 and 1 use absent published payload and do not write the JOC complexity field.

The native capture names its field `complexity_index_type_a`; the approved F12 wire contract names the selected field `complexity`. Both are 8-bit JOC additional-BSI concepts, but the approved material does not bind the capture-local configuration record to the selected F12 receipt or AU. The captured value 16 and model value 16 do not establish that join.

## Independent domains

- Configuration count: signed I32 from rate/channel-label configuration; parameter ID 13 and query selector 10 belong to this producer namespace.
- Wrapper profile pair-list key 13: a separate profile namespace with unresolved `X + Y`; it is not configuration parameter 13.
- ID14 `downmix_index`: separate EMDF field (selected value 3); it does not define the count.
- ID14 `object_count` and ID11 `slot_count`: EMDF semantic counts consumed by the selected F12 model rule.
- `carrier_block` and transport `first_block`: separate domains outside the bounded count capture; this crosswalk establishes no relation to either.

The selected ADM metadata receipt establishes 38-channel format, DirectSpeakers/Objects labels, and 38 CHNA track indices. It does not map those facts to the native profile pair list, selected profile-row identity, `a7`, partial-block flag, or list selector. The DBMD payload was not inspected, and the selected wrapper invocation was not captured.

## Evidence grades and remaining boundary

- Count query → constructor → BSI producer → importer: `HASH_BOUND_CAPTURE_LOCAL_JOIN`.
- Native field location: `HASH_BOUND_CAPTURE_LOCAL_JOIN_PLUS_EXISTING_WIRE_FIELD_SUPPLEMENT`.
- Selected F12 object-count-to-complexity rule: `FROZEN_APPROVED_MODEL`.
- ADM metadata → native wrapper/profile input: `UNBOUND`.
- Captured configuration record → selected source/frame/AU F12 consumer: `UNBOUND` (`NO`).
- The Python replayer source identity is recorded, but its code is not included and it was not executed for the selected-profile/AU join audit.
- Native execution, selected per-AU count production, full-song replay, and product acceptance: `NOT_RUN`.

No numeric equality, shared module image hash, event order, or local record alias is used as an identity join. No source code, disassembly, machine code, addresses, PCM, or private key is included.
