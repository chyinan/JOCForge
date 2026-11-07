# RF snapshot +0xA4 branch-to-worker input boundary

## Three bounded static branches

`FUN_1801764e0` selects one of three constructed receiver wrappers from snapshot fields `+0x9C` and `+0x28`:

- `+0x9C==0 && +0x28==0`: `FUN_1801850a0` → slot `+0x18` `FUN_180186520` → first worker `FUN_180186870`.
- `+0x9C==0 && +0x28!=0`: `FUN_180185100` → `FUN_180186550` → `FUN_180186580`.
- `+0x9C!=0`: `FUN_180186e40` → `FUN_180189490` → `FUN_1801894c0`.

Each constructor stores an inner-initializer return at wrapper `+0x08`. Each slot target passes that `wrapper+0x08` value as the first argument to its worker; the decompiled worker formal is generic 64-bit `longlong param_1`. The machine-level argument therefore is a constructor-produced inner-state pointer, not a proven scalar from the snapshot.

## Remaining first break

The factory's bounded body has no direct `+0xA4` access, and the first worker receives no explicit snapshot `+0xA4` scalar. Whether one of the earlier opaque helpers (`FUN_180182850` or `FUN_1801431f0`) indirectly copies that field into the constructed inner state remains `UNBOUND`. The runtime branch and receiver class are `NOT_CAPTURED`. No global no-consumer claim is made.

The next narrow edge is the selected slot target's `wrapper+8` inner-state argument to the first worker's typed fields. Continue only while the same snapshot owner can be followed; stop when the owner chain is lost.

## Provenance

Upstream manifest SHA-256 `6761ae7bd37ff11ce25470c9c61c2d0313c98214d5a19d63a2cd444d4e0d778a`; upstream package-seal SHA-256 `6c81aa388e423ef2e2b548433fdfe964c3c3685f65ec03b339d350bfbde88b7f`. Normal and `python -O` source checkers passed. The supplement includes the sanitized branch matrix and hashes only, not source scripts, traces, binary, media, PCM, payload, key, or license.
