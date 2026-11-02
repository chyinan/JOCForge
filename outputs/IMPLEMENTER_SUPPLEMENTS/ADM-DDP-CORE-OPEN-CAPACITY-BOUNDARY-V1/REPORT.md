# DDP core open-configuration count boundary

**Status:** the scoped `DdpEncoderApp::open` writer path is bound to one descriptor group and per-group capacity 11; its relation to ADM rows or the selected Atmos JOC CLI remains unbound.

## Bound open-path configuration

In the bounded `DdpEncoderApp::open` path, `FUN_1402142d0` initializes descriptor-group count to 1 and inserts the explicit key/value pair `(2, 11)` for all accepted encoder modes 0, 1, 3, and 4. `FUN_1403162b0` rejects any group count other than 1 with `0x1A`. `FUN_14031fae0` writes the group count to `root+0xC138` and resolves explicit key 2 to `root+0xC140`; the profile default fallback is bypassed.

`FUN_140315db0` rejects descriptor-group count > 1 with `0x14`, and rejects a per-group count > 11 with `0x1D`. If the count passed to that per-group gate were directly 36 or 38, this specific open path would reject it.

## Critical limit

The package does not bind ADM CHNA/physical tracks, Atmos plugin six-byte rows, `code13-code14`, or the Rust CLI's codec source count to the core `frame+0x20` per-group count. Therefore **no 36/38 rejection observation** is claimed for the selected JOCForge/Atmos path. The capacity of this API open path must not be described as a global ABI limit or copied into JOCForge without closing that typed route. The package also does not prove every core API caller uses the same `DdpEncoderApp::open` configuration.

The smallest next edge is a same-run typed join from the selected Atmos route to this core frame-view input and its open-config owner. Until then, keep the native frame-count source `UNBOUND` and do not conflate it with the plugin's separately observed 1–128 packet-row gate.

## Verification scope

Wash reran the source package verifier in normal Python and `python -O`; both passed for five listed payloads. This was Ghidra 12.2 static-only: no DEE run, media/payload/PCM/key/license read, or implementation contact.
