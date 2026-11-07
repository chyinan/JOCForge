# DEE range record to AtmosReader cursor boundary (current source snapshot)

## Disposition

This Wash supplement supersedes `DEE-RANGE-RECORD-TO-FRAME-READER-BOUNDARY-V1`. The source directory has a newer current manifest (`250b...`) than Agent9's prior final message (`1187409...`); Wash rehashed the current manifest and all listed artifacts, and the adjacent `MANIFEST.sha256` matches the current manifest. The current checker receipt reports normal and `python -O` PASS. No formal package seal file is present.

## Bound range-to-reader path

The `file_position`/`embedded_timecode` split in `FUN_18014fd90` feeds the prior range-item builder. In the selected mode-1 encode path, `FUN_180175500` iterates range records and passes each record/source index into `FUN_1801764e0`. That helper builds a per-source reader descriptor and forwards it through `FUN_1801852e0` to `FUN_1801828c0`, which consumes 0x40-byte descriptors and calls `FUN_1801831c0`; the constructed source object is stored in `AtmosReaderPrv+0x8`. This binds the range record to Reader input construction.

Two concrete vtable slot `+0x18` owners are identified:

- When per-source byte `+0x140` is zero, `FUN_1801850a0` constructs `AtmosReader`; its slot `+0x18` target is `FUN_180186520`, followed by `FUN_180186870` and `read_float` at RVA `0x18692f`.
- When byte `+0x140` is nonzero, `FUN_180185100` constructs `AtmosReaderCC`; its slot `+0x18` target is `FUN_180186550`, followed by `FUN_180186580` and a queued source-item read at RVA `0x18669e`.

For the AtmosReader branch, the private cursor is at `+0x28` and total at `+0x20`; the cursor is initialized to zero and checked against total before the first float read. The runtime `+0x140` value is not captured, so which Reader is selected remains `UNBOUND`.

## Remaining timebase edge

The scoped Reader methods do not retest the `file_position`/`embedded_timecode` label; that selection is upstream. While the range record reaches Reader construction, the start/end value-to-inner cursor `+0x28` or total `+0x20` mapping is not established. The upstream start field `+0xf98` join, timebase units, exact trimming behavior, and runtime Reader owner remain `UNBOUND`/`NOT_RUN`. No claim of timebase parity or trim equivalence is made.

## Limits

- Input-only `file_position` and synthetic `embedded_timecode` vectors are `NOT_RUN`.
- No DEE run, WinDbg trace, media/PCM/payload/key/license access, or implementation modification.
- No raw Ghidra scripts, logs, or decompilation were copied into this supplement.

## Implementation scope

Use this as a static range-record-to-reader construction contract. It identifies two reader owners and their first-read interfaces, but does not map XML start/end to the actual sample cursor or prove which mode runs. Continue from the reader cursor/total producer and the `+0xf98` typed join.

## Current source package

`C:/Users/chyinan/Documents/Codex/2026-11-01/jocforge-reverse-agent-9-f05-f13-static-lineage/outputs/dee-range-reader-consumer-wash/`; current manifest SHA-256 `250b3594fe21b7f5e6883705d4cc68ac5d6655acca97ebb3bf1093cecc5d9515`; `CHECK-RESULTS.txt` says normal and `python -O` PASS. This replaces the earlier reported manifest hash `d6d9f5b...`.
