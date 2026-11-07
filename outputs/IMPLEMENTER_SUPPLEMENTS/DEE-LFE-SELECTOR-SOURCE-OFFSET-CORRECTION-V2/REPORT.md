# LFE selector source offset correction V2

## Register-joined source correction

The hash-bound Ghidra 12.2 register trace follows `FUN_140350d00` passing the child row pointer `lVar9` in `RDX` to `FUN_140317600`. At entry, `R14` receives `RDX`; the call to `FUN_1403471f0` uses `RCX=R14+0x60`. Since the helper copies `param_1+0x34/+0x38` to core state `+0x2c/+0x30`, the effective child-row source fields on this path are `lVar9+0x94/+0x98`, not the earlier `+0x40/+0x44` claim. The child `+0x40` qword reset in `FUN_140350ac0` is a different field.

## Same-owner writers and unbound field semantics

The dispatch receipts show the same `channel_instance` reaching method-table slots `+0x20` (`FUN_140350ac0`) and `+0x28` (`FUN_140350d00`), using the same child array. In the static initialization path, `FUN_140350ac0` calls `FUN_140351c90`, which first writes child `+0x98` from descriptor entry ID 5, then calls `FUN_14031ff50`, which later writes child `+0x94/+0x98` from the configuration object at `channel_instance+0x1098`.

`FUN_14031ad40` reads config `+0x64` as U32; values below 8 are written directly to child `+0x94`, while values 8 or above use a static mapping table. `FUN_14031af20` reads config `+0x70` as I32, maps -1 to 0, and writes child `+0x98`. Thus the later writer supersedes the earlier descriptor-ID-5 assignment on this path. The config pointer producer and semantic labels for `+0x64/+0x70` remain `UNBOUND`.

## Crosswalk boundary

The source fields are now byte-offset bound for this static path, but their relation to ADM CHNA rows/layout and to any matched run values is `UNBOUND/NOT_CAPTURED`. No LFE root cause or numeric parity is established. This correction supersedes the old `lVar9+0x40/+0x44` selector-source claim; it does not change the separate finding that the two reviewed LFE lanes set recurrence selector zero, nor the independent instruction-order facts.

## Provenance

Upstream manifest SHA-256 `46473d85233accc9a6a98d00516f2910023175437fc03e28a202c0e67868c2ed`; upstream `MANIFEST.sha256` file SHA-256 `e72d4362f5a265b90b6663ac7414ad809423eb81a9fb65027e438a22ff8ba36c`. Normal and `python -O` source verifiers passed. Source seal/source identities and referenced parent receipts are preserved as hashes; Ghidra scripts/logs, binary, PCM, payload, key, and license contents are not included.
