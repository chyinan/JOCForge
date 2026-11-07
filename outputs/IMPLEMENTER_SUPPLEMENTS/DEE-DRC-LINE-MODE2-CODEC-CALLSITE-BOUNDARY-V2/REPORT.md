# DEE line-mode mode-2 codec-callsite boundary

## Disposition

Wash verified the source package manifest, all six payloads, package seal, parent mode-2 package, and target DDP image. This package advances the mode-2 inspection to the first codec-processing callsite; it still does not connect line DRC record `+0xa0` to codec configuration, process arguments, or final BSI.

## Bound selected mode-2 path

The selected mode-2 branch is `Priv+0xfc8 == 2`: `FUN_1801746c0 -> FUN_1801746d0`. The mode-2 loop retains the per-frame record and constructs an `EmpireEnc` wrapper from record subobject `+0xd8`. The inspected constructor copies config fields only through that subobject's `+0x5c`; it does not receive record `+0xa0` on this edge.

At callsite RVA `0x174fc7`, `FUN_1801746d0` invokes the `EmpireEnc` vtable slot `+0x8` (`FUN_180179a80`) when the block descriptor size is `0x600`. The wrapper forwards processing arguments to `FUN_180179ab0`, which accepts a `0x600` processing descriptor or zero-size flush, assembles codec-process inputs from the current block and implementation state, and calls `FUN_1801a8840`. It then packages the produced byte buffer; the caller passes that buffer to `FUN_180189c70`.

This binds the first mode-2 codec call and immediate byte-buffer writer. No typed transfer from the source record `+0xa0` into the per-item `+0xd8` config, codec-process arguments, or output BSI is established in this slice.

## Limits

- The provider-rooted line StringType handler and runtime mode selection remain `UNBOUND`/`NOT_RUN`.
- Final DRC BSI/audio effect remains `UNBOUND/NO`; no global absence claim is made.
- No DEE execution, media/PCM/payload/key/license access, or implementation change.

## Implementation scope

Do not implement a line-mode DRC rule from the mode-2 codec callsite alone. The static boundary shows mode-2 reaches codec processing but does not establish that the line profile controls it. The next exact edge is inside `FUN_1801a8840`/its direct selected consumer: determine whether any typed input from the mode-2 config corresponds to record `+0xa0`, then stop if no lineage exists.

## Source package

`C:/Users/chyinan/Documents/Codex/2026-10-23/jocforge-reverse-agent-3/outputs/adm_bwf_ddp_drc_line_mode_mode2_codec_callsite_boundary_wash_v1/`; manifest SHA-256 `fb34f4f2a02f11b0f55c77eefc12adca04fcf6a919a73df4dc66c6b2e6f84827`; package seal SHA-256 `3d03cfc3f059ab6b977990704c4d65adb880cd920653f1400d12bbc31c492517`.
