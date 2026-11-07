# DEE measure_only to DDP owner-mode boundary

## Disposition

Wash verified the source package manifest and six payloads, its seal, and both target image hashes. This binds the host StageRunner property walk to an indirect stage setter, and separately binds the DDP plugin's property-map setter and mode-writing vtable method. It does not join the host object to the DDP `FilterDDPJocEnc` instance or the `measure_only` value to that instance's mode field `+0x8`.

## Host property path

The host `dee.exe` path constructs a target stage object through `FUN_140041be0`, stores it at StageRunner `+0x280`, then calls `SetStageProperties` (`FUN_14004d610`) with the stage object and property rows. Rows carry key at `+0x18` and value at `+0x38`; eligible non-storage rows reach `FUN_140054010`, which invokes an indirect vtable slot `+0x8` at `0x14005421f`. The host image does not resolve that dynamic target to the DDP plugin vtable.

## DDP plugin side

`FUN_180136220` registers `measure_only` and nested `loudness:measure_only` properties in configuration member `+0xad0`. `FilterDDPJocEnc` slot 1 is property-map setter `FUN_18013e9d0`; it calls an indirect property callback at slot `+0x28` and does not directly store the adapter's `+0x8` field. A separate slot-5 method `FUN_18013e920` reads its own `this+0x8`, writes U32 to owner `+0xf34`, and calls `FUN_18014a600`.

The two sides remain unjoined: host StageRunner slot target/object to DDP plugin instance is `UNBOUND`, and the `measure_only` property callback to adapter `+0x8` is `UNBOUND`. Therefore `measure_only=false/true` is not mapped to owner mode `0/1`. The prior static mode semantics (`owner+0xf34==0` measurement; `==1` normal encode) remain separate evidence.

## Limits

- Input vectors are symbolic and `NOT_RUN`; host wire type and runtime adapter target are not established.
- No DEE process, media/license/auth access, dynamic trace, or implementation-source change.
- Host and plugin Ghidra databases are separate; unchanged database hashes do not create a cross-module object join.

## Implementation scope

Do not map `measure_only` directly to DDP owner mode based on the two booleans or similar names. The next exact static edge is the host stage object's selected vtable target and the DDP property callback that might write adapter `+0x8`; retain either as `UNBOUND` until a typed transfer is found.

## Source package

`C:/Users/chyinan/Documents/Codex/2026-10-25/jocforge-reverse-agent-4-f06of/outputs/measure_only_owner_mode_wash_v1/`; manifest SHA-256 `665912add98deaa7c276f728ee8301e8a7cbe571f985fb0ab7570bc8a420aa7f`; source seal SHA-256 `cd1f4a31423e51283a3fad7cc7ee1ea9486f0a3879430adaa42a08ef33b69f4e`.
