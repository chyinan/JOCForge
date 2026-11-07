# DI bool property receiver boundary

## One bound receiver edge

The hash-bound Ghidra 12.2 static trace follows `SimpleType<bool> this` through `FUN_18014f640`: the entry receiver in `RCX` is saved in `RBX` and restored to `RCX` before the indirect call through receiver slot `+0x28` at RVA `0x14f68f`. For this vtable, the target is setter `FUN_18016a540` at RVA `0x16a540`. The same receiver therefore flows through this wrapper-to-setter edge.

## First break and scope

The generic property-map caller that supplies the receiver to `FUN_18014f640` remains `UNBOUND`. A previous bounded `+0x28` scan hit its 100-candidate cap; the StageRunner Ghidra project could not be opened because it was locked. The receiver is not joined to `Priv+0x274`, measurement root `+0xf6c`, or source record `+0x114`; owner and run generation remain `UNBOUND`.

The parent receipt binds a parsed-bool write to `SimpleType<bool> this+0x50`, but the first measurement/audio consumer of that field is `NOT_IDENTIFIED`. No DI A/B, audio read, runtime owner/generation capture, or DEE run was performed. The package's old UI runs used `D:\ADM.wav`, not one of the user-authorized samples; those are not sample-pass evidence and are excluded from the result. No process control or retry is implied.

The exact first break is the generic map lookup that produces the receiver. This receipt closes only pointer continuity inside `FUN_18014f640 → FUN_18016a540`; it does not establish DI effect or the DI product path.

## Provenance

Upstream `MANIFEST.sha256` file SHA-256: `9bb2cd833efdd70256b42c5c52689010a84eed02ab3b47ad5b12abf3a1168971`; upstream package seal SHA-256: `3d7ab162b3b6623178e42f0d9a4a7adf878c6a76c724fb043094738b672c2772`. Wash recomputed every listed upstream file hash and verified all three parent manifest/seal pairs. The upstream Ghidra script, logs, and trace are not included here; only sanitized findings and parent hash references are transferred.
