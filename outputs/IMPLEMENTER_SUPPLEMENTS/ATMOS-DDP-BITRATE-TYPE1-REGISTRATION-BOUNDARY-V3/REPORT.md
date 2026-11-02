# Atmos DDP type-1 writer registration boundary

**Status:** rate values reach the type-1 writer slot; no in-image static type-1 registration writer was located in the inspected paths; runtime callback target and named BSI/frame field remain unbound.

## Bound path and loader audit

Earlier Ghidra traces bind codes 4/5 to rate integers 768/1024 under field ID 3 in the core table, which the serializer dispatches through the type-1 callback table's `+0x28` slot. In the static image, the type-1 callback-table pointer cell at RVA `0x652218` is zero and lies in initialized, non-writable `.rdata`.

The V4 follow-up inspected the PE entrypoint, TLS callback directory, bounded DllMain path, named exports, delay imports, relocation entries, and the prior bounded CRT initializer result. No TLS callback was present; the indirect DllMain callback cell was zero; no direct type-1 cell reference/store was found in the inspected initialization path; no registration export or relocation to the type-1 cell was found. The nine direct registry-base xrefs contain no STORE P-code. The type-10/11 relocations and shared callback do not identify type 1.

This is a bounded static search, not proof that the runtime cell stays null. External/runtime registration remains **UNBOUND**. The actual type-1 callback target and field-ID-3 handling are not named.

## Remaining output/audio boundary

The 768/1024 integer configuration path is bound, but field ID 3 is not mapped to a named E-AC-3 BSI, frame-size, or output-rate field. Actual bitrate, frame size, and decoded-audio impact are **NOT_RUN / UNBOUND**. Do not infer full-rate support or audio equivalence from the static cell value.

The next smallest observation is to watch module RVA `0x652218` from the earliest DLL-load event through attach/registration, then inspect `R10` and `[R10+0x28]` at the first type-1 dispatch. No DEE process was run for this pass.

## Verification scope

Wash independently verified the V4 manifest SHA-256 and all 29 listed evidence files; no unlisted file was present. The source package has no separate `package_seal.json`. Its target image hash is carried forward from prior evidence and was not rehashed by Wash in this pass. Ghidra scripts, raw evidence text, and binaries are excluded from this supplement.
