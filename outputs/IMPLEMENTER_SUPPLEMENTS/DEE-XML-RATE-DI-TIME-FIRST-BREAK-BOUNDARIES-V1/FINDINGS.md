# DEE rate, DI, and ADM time first-breaks

Package scope: sanitized Wash handoff of four independently verified static slices. It records bounded implementation cautions and open edges; it does not provide a replacement algorithm or assert end-to-end DEE equivalence.

## Rate to carrier writer

The sealed DEE static slice binds selected type-1 field ID 3 to `FUN_1801b1590` and a registry row. The callback cell at RVA `0x652218` is zero in the inspected static image; the bounded searches did not find its runtime registration. The callback target and the join from field ID 3 to a named E-AC-3 `frmsiz`/rate writer remain `UNBOUND`; native carrier-rate/frame-size output is `NOT_RUN` in this slice. This does not invalidate JOCForge's independent product rate path or the six washed frame-header observations.

## Host data_rate property transfer

The host package statically binds one StageRunner build branch: it matches a route name, constructs a stage instance from the same candidate config, retains both objects, then forwards the property vector through the stage's generic setter. The specific `data_rate` record, selected Stage's identity as Atmos DDP, and single-token versus multi-token handling remain `UNBOUND`; no DEE run was performed.

## DI to inner gate

The selected inner path accumulates per-channel state bits into a DWord and passes it to the first numeric consumer. The outer DI Bool at subcontext `+0x164` is not directly forwarded on the inspected path. The typed source/owner/generation join from that Bool to the inner gate word remains `UNBOUND`. A reused Human Cat A/B is a single-input observation only; no new A/B or spatial render was performed.

## ADM time token to atmos_info Start time

In the inspected `atmos_info.exe` image, an ADM time-token is parsed to a double, written to descriptor `+0x60`, then reported as `Start time (in seconds)`. The exact XML element/attribute and absolute-versus-relative timebase remain `UNBOUND`; descriptor `+0x58` FFOA source also remains `UNBOUND`. This is the separate `atmos_info` report path, not the `encode_to_atmos_ddp` output-range control.

## Handling

Treat each row as its own native static fact and first break. Do not infer missing callback registration, plugin identity, XML lineage, runtime generation, BSI effect, audio equivalence, or JOC spatial rendering. All source artifacts are hash-bound by `SOURCE-RECEIPTS.json`; this handoff includes no Ghidra source/decompilation, binaries, raw ADM, PCM, EC-3 payload, key, or license content.
