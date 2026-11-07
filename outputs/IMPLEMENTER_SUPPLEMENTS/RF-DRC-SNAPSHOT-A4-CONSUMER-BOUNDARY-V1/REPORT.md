# RF DRC snapshot +0xA4 consumer boundary

## Bound static facts

The RF-only receipts bind the `rf_mode_drc_profile` StringType value through the exact profile-tree lookup to a raw U32 snapshot write at `FUN_1801535b0.param_2+0xA4`.

A distinct RF configuration path starts from snapshot `+0xE0`: mode-1 selection reaches `FUN_1801754f0`, then `FUN_180178dc0.param_1[2]`, conversion by `FUN_1801787e0`, and typed encoder configuration `DDP_COMPR` U32 at config `+0x24` via `FUN_1801aa730`. These are two fields in one snapshot record, but no same-owner copy, alias, or dataflow from `+0xA4` to `+0xE0` or to the `param_1[2]` consumer is established.

## Exact first break

The unresolved edge is `FUN_1801535b0.param_2+0xA4 → first typed RF consumer/owner`. The next bounded check is to follow only the same snapshot owner from `FUN_1801535b0`'s direct caller to its first post-write consumer and test for an exact `+0xA4` read or copy. Keep the separately proven `+0xE0` route distinct; equal offsets or nearby fields do not establish identity.

Runtime RF value, snapshot contents, selected per-frame route, and encoder-config value remain `NOT_CAPTURED`. No conclusion is made that `+0xA4` has no consumer elsewhere. This evidence does not resolve the RF DRC audio effect and does not justify a product mapping.

## Provenance and scope

Upstream manifest SHA-256: `b6f83de105efbb41ec476746b424053d96807d6b218cfd2caf68b5223e576967`; upstream package seal SHA-256: `4f3c853e754d9b1fbcf868921a4d81891d0ce94c3529529c655cca70cb17e68c`. Wash independently recomputed the manifest and package-seal bindings, all five payload hashes/sizes, `SHA256SUMS`, and each listed source-artifact hash. No line-mode `+0xA0` work, global xref sweep, DEE run, PCM/EC-3 payload/key/license read, or implementation access occurred.
