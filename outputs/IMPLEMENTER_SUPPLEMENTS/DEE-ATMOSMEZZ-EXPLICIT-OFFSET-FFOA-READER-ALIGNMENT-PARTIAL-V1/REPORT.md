# Explicit offset and FFOA: partial Reader boundary

## Scope

This Wash supplement records only the statically observed explicit offset and ffoa conversion path and the first generic Reader sample-read boundary. It is evidence for a partial XML contract, not a complete or production-ready time-axis implementation.

## Statically observed behavior

- In Ghidra 12.2, the explicit non-auto values are passed with timecode_frame_rate to the same converter. The converter returns seconds as a double.
- A dotted value is parsed as hours, minutes, seconds, and a decimal fractional-second suffix. The suffix is parsed as 0. plus the suffix. This is a fractional-second representation, not a claim that the suffix is an integer number of milliseconds.
- A value without a dot follows the frame form: a bare base-10 integer is interpreted as a frame index; the colon form contains hours, minutes, seconds, and frames. A trailing lowercase df selects a drop-frame correction branch.
- The explicit offset replaces the selected source offset after the source offset map has been consulted. The explicit FFOA branch does not read the source FFOA map; the final FFOA is clamped to max(selected_offset, converted_explicit_ffoa).
- The DDP source setup reads offset and FFOA separately. Its first-frame-of-action path subtracts offset from FFOA before passing the relative value to a helper.
- A reused Reader receipt identifies the first generic read_float in FUN_180186870; its private cursor at +0x28 advances by configured frame size +0x10.

## Open boundary

No typed edge joins the normalized explicit offset/FFOA values to the Reader cursor's initial value, a seek, or duration clipping. Therefore sample alignment and audible timeline effect remain UNBOUND. Runtime Reader variant and same-run owner/generation were not observed.

The ADM start-time candidate is parsed to a double and reaches descriptor +0x60 and master +0xA8, but its exact XML field and time origin are UNBOUND. The FFOA candidate is read at descriptor +0x58 and reported through master +0xB0; its ADM/BWF producer and source chunk remain UNBOUND.

The reused seven-file metadata scan found no distinguishable source offset/FFOA candidate. The proposed auto versus explicit-FFOA contrast is PROPOSED_NOT_RUN: it has no assigned paths, no launch authorization, and no result.

## Boundaries and handoff status

IMPLEMENTATION_READY=NO. This supplement must not be used to claim Reader alignment, XML runtime admission, full time-axis support, or an executed A/B result. The next closure is a typed source/Reader-position join or an authorized input with a distinguishable FFOA candidate, followed by a separately approved minimal experiment if static evidence is insufficient.

No DEE or atmos_info run was performed for this slice. No media payload, PCM, key, or license was read or retained. The evidence contains no source code or decompilation.
