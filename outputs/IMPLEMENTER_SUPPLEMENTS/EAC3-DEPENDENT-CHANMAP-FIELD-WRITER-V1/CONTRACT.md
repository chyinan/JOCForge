---
status: implementer_ready_limited
topic: "E-AC-3 dependent-substream chanmape and chanmap field pair"
source_manifest_sha256: 0c429c3a6f4da005458e8e951351b77008e900db9509cd0bb9f916dd411ef6d4
profile_manifest_sha256: 7876e3c53e7bda56f60c8d3a691f9082925c6ef14a7899cff21c09f8b1416714
confidence: ETSI-bound field syntax plus user-target map
tags: [E-AC-3, chanmape, chanmap, Flat7x, bit-writer]
---

# E-AC-3 dependent chanmap field pair

## Implementer slice

Add an additive pure serializer in the existing eac3_channel_map responsibility (or a same-boundary helper) that emits the D0 target's chanmape=1 bit followed by the typed 16-bit chanmap value. The caller owns the BSI cursor and all surrounding fields. The existing 16-bit numeric field helper remains available for callers that already serialized chanmape.

Preflight the full 17-bit field group before writing any bit so insufficient capacity leaves the bit writer and storage unchanged. Write the map MSB-first. The target input is DependentChannelMap(0x1A00), which sets MSB-indexed locations 3, 4, and 6: Left Surround, Right Surround, and the Lrs/Rrs pair, yielding four coded locations. Its validation context is acmod=6, lfeon=0.

The full 17-bit target sequence at bit alignment zero is 1|0001101000000000 (chanmape, then the 16 map bits). The target row's I0/D0 order, rate fields, six-block frmsiz, D0 frame creation and source-role assignments remain separate typed domains.

## Evidence and limits

ETSI TS 102 366 V1.4.1 (2017-09), clauses E.1.3.1.7/E.1.3.1.8, specifies a one-bit chanmape for dependent substreams and a following 16-bit custom map when enabled. E.2.8 states that enabled map locations define dependent-substream coded-channel order. Table E.1.4 indexes bit 0 as the MSB and maps bits 3, 4, and 6 to Ls, Rs, and the Lrs/Rrs pair. The target 0x1A00 and acmod/lfeon context come from the user-approved Flat7x profile contract.

This slice authorizes pure field serialization and input-only vectors. It leaves the DEE record +0x5c owner/source, connection to a native D0 BSI writer, actual D0 audio generation, runtime generation, frame/AU assembly, and production_allowed status unchanged. This helper does not change the existing 5.1 resolver or select a 7.1 production path.

## Validation requested

Use the input-only vectors to cover all 16 one-hot masks, the target map, misaligned cursors, and the insufficient-capacity no-partial-write case. Run the existing 5.1 rate/profile regressions as part of the implementation change. Keep the full Flat7x output gate closed until the separate D0 generation and source-role blockers have their own evidence.
