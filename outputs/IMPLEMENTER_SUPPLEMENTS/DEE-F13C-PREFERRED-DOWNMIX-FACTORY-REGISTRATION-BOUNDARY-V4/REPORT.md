# DEE preferred-downmix factory registration boundary (current)

## Disposition

This supplement supersedes earlier preferred-downmix owner-transfer boundary supplements. Wash verified the source manifest and five payloads, SHA256SUMS, three related F13C/ADM packages, and the installed `dee_ddpjoc_encoder.exe` hash. The source package has no formal package seal file.

## Bound owner/type and field transfer

Ghidra RTTI binds vtable address point `0x140bca558` to `Factory<AtmosMasterFileStorage>`; its class-hierarchy descriptor lists that type and `PrintmasterStorageFactory`. Slot `+0x08` targets `FUN_1403c3980` and sibling slot `+0x10` targets `FUN_1403c3d30`.

The selected slot `+0x08` path forwards opaque `param_1+8` through `FUN_1403f5270` to constructor `FUN_1403f1dd0`, which stores it in callback descriptor `+0x6368`; the callback reads it. The sibling slot `+0x10` method also reads the same factory `+0x8` and forwards it to a separate descriptor-like path. This binds an opaque factory-owned selector transfer, not its semantic field name or value.

## Factory registration remains the break

The imported program's reference manager reports zero code references to the vtable address point `0x140bca558` and its COL. No constructor or registration callsite in this imported program was found that writes this vptr and initializes factory `+0x8`. The selector's field type/value is therefore unknown. Nothing joins it to the selected reconstructed job XML value `preferred_downmix_mode=loro`; the `dmixmod` parser is a separate string validation path with no typed `ddp_dmixmod` consumer.

## Limits

- The LoRo source XML is a reconstructed fixed job template, not runtime capture; input vectors are `NOT_RUN`.
- 36/38 ADM references are metadata-only; no audio `data` was read.
- Main JOC audio, stereo fold-down, BSI/metadata effect, compatibility hint, intentional ignore, and spatial-object rendering remain `UNBOUND`/`NOT_COVERED`.
- No DEE/WinDbg/A-B, payload/key/license access, or implementation change.

## Implementation scope

Do not map `preferred_downmix_mode=loro` to the opaque Factory selector or `dmixmod` by adjacency or name. The remaining exact edge is the constructor/registration that writes vtable `0x140bca558` and initializes factory `+0x8`, possibly outside the currently imported program. No default, error, or effect is inferred.

## Source package

`C:/Users/chyinan/Documents/Codex/2026-11-01/jocforge-reverse-agent-8-f13c-preferred-source/outputs/F13C-preferred-downmix-factory-instance-registration-boundary-v1/`; manifest SHA-256 `06576a27c87f4758d2edbc22fa20c58edae981858bd0223c08295fd7d1a8111d`; SHA256SUMS SHA-256 `2a1846165d6abb65fb6274504570a7f61d9a439710adb72221a0b578b6969763`.
