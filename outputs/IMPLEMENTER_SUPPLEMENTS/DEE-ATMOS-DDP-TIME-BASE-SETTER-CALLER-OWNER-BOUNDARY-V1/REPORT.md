# DDP time-base setter caller-owner boundary

Wash independently checked the sealed caller-slice package, all four payload hashes, its checksum inventory, and the three cited parent manifest hashes. This is Ghidra 12.2 read-only/no-analysis static evidence, limited to one caller.

`FUN_18014f9f0.param_1` is identified in the reused chain as the nested stage owner at `P+0xb8`; the caller copies it separately to `local_1c0`. The argument passed at callsite `0x18014fc94` to `FUN_180146180` is `&local_1e0` (`RCX=RSP+0x38`). That stack temporary is constructed from `local_160` through `FUN_180157ea0` and populated through `FUN_18012ca90`. The bounded caller slice shows no assignment, alias, or address formation from `param_1+0x550` into `local_1e0`.

This does not establish a typed join between the temporary and the nested time-base descriptor at `+0x550`; the temporary's relation to a typed option descriptor/writer, and the XML value-or-omission path to `+0x550`, remain `UNBOUND`. No callee body beyond the already-bound caller path was inspected. No DEE job ran; no media, PCM, payload, key, or license was read. This is not a global absence or product-readiness claim.
