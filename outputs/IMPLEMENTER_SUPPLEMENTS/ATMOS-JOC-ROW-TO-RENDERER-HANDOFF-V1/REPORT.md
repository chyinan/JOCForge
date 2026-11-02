# Atmos JOC planar row handoff to renderer

## Scoped rule

The hash-bound `dee_audio_filter_ddp_atmos.dll` path supports a direct planar row handoff into `AtmosProcessor::process` and the concrete `SinglethreadSymphonyRenderer` process slot. For the supplied descriptor, pass one row pointer per channel in source row order, an explicit channel count, and a frame count. The processor checks that the channel count matches its configured count and that `frame_count` fits the allocated capacity; on a non-empty frame it copies `frame_count * 4` bytes from each source row to the matching `MultiChannelBuffer` row, then dispatches renderer process through vtable slot `+0x08`.

This is a ready input-plumbing rule. Do not interleave or permute rows at this boundary. The nested renderer analysis/QMF target remains `UNBOUND`, but that does not block implementing the descriptor checks, index-preserving row copy, or renderer dispatch.

## Confidence and sample scope

Confidence is high (`0.95`) for descriptor checks, copy span, and row order; medium-high (`0.82`) for this being the selected route's best-supported core path. The route chain combines hash-bound static Ghidra edges and two prior authorized ADM receipts, each 36-channel / 48 kHz / 24-bit on `audio.adm -> encode_to_atmos_ddp`. One prior receipt contains an active source window; the other is a quiet-window result and is excluded from active-audio claims. Both report encoder completion and JOC metadata. No new DEE run, render, or waveform comparison occurred in this slice.

The target `人是猫` input is 38-channel; this supplement contains no 38-channel runtime render. Use its real channel count and the separate seven-input CHNA row-order rule; do not treat the two 36-channel receipts as proof of a 38-channel full-profile encode.

## Audio boundary

The row copy is four bytes per sample at the AtmosProcessor input boundary. F32 conversion/quantizer semantics belong to the separate coefficient-payload slice and are not inferred here. The exact nested audio-analysis/QMF callback behind renderer helper vtable `+0x28` remains `UNBOUND`; no sample-exact or whole-track audio-equivalence claim is made.

## Input-only vector and exclusions

`INPUT-ONLY-VECTOR.json` is structural only: 36 row pointers, matching channel count, frame count 512 within capacity 512, 16-byte pointer alignment, and no sample values. It was not executed. No source code, DEE binary, disassembly, decompilation, audio, PCM, key, or license material is included.
