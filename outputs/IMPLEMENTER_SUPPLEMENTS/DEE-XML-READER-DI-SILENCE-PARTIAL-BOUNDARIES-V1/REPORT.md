# DEE XML partial boundary receipts

This supplement records three independent, sealed Wash findings for evidence-registry updates only. It is not an implementation specification. `IMPLEMENTATION_READY=NO`; no audio algorithm change is authorized by these receipts.

## DI Bool

Source package `di_dialogue_intelligence_property_measurement_boundary_wash_v1` is `SEALED_PARTIAL`. Its conditional static path places the registered `dialogue_intelligence` Bool candidate into measurement setup byte `+0x94`, then passes that setup toward the per-record measurement boundary. The queried runtime record name and owner/generation are not joined to that Bool. A numeric-result or final audio-control dependency is also unbound. Therefore DI true remains unsupported; do not infer an effect.

## start/end range handoff

Source package `DEE-ATMOS-DDP-COMPOSITE-OPEN-DURATION-HANDOFF-WASH-V1` is a six-payload static Ghidra receipt. `CompositeMasterItem.startSample/endSample` arrive in the item-list pointer passed as RDX to `FUN_180550800`; the caller stores the newly allocated duration object pointer at `CompositeMaster+0x50`. The callee's item-list consumption, duration-object contents, and concrete `+0x60` duration method remain unbound. This closes only the direct handoff edge, not seeking or range processing.

## silence builder reference boundary

Source package `dee_atmos_ddp_silence_builder_xref_wash_v1` is `SEALED_PARTIAL`. In the existing Ghidra reference database, `FUN_1801844c0` has three references, including one direct call at `0x175e10` in `FUN_180175c80`; the other two references are unclassified. No additional direct caller was found in that bounded set. This is not evidence of silence-duration conversion or a global absence of consumers; duration/audio effects remain `NOT_RUN`.

## Scope and handling

All three source packages passed their checker in normal and `python -O` modes. This supplement copies no source, decompilation, PCM, media, payload, key, or license. The three findings are kept separate; no cross-package owner or runtime identity is inferred. Keep open boundaries as `UNBOUND` or `NOT_RUN`.