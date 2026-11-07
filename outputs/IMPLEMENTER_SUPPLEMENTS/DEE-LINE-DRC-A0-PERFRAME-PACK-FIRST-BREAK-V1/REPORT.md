# Line-mode DRC value to first per-frame packer boundary

## Bound static path

The upstream package reuses the sealed StringType-to-snapshot receipt: the line-mode property result is a U32 at snapshot+0xA0. The same snapshot pointer reaches the per-frame path: a mode-1 call passes a stack pointer through a thunk that preserves RDX, then into the downstream function as the snapshot base. That function passes the same pointer to a validator and to a per-frame packer.

The bounded validator and packer inspections contain no direct snapshot+0xA0 field read or copy. The packer reads other snapshot fields and selected row/vector pointers. This is the first unresolved codec/BSI handoff boundary for the line-mode value.

## Limits

Indirect helper behavior, the value's codec/BSI representation, runtime mode, record identity/generation, and clock remain unbound or uncaptured. The package makes no global absence claim and does not establish an audio effect. DEE was not run; no PCM, payload, key, or license data was read.

Wash independently verified the upstream manifest, all seven payload hashes, SHA256SUMS, and package seal. The upstream agent reported normal and python -O checks passing; no checker executable was included in the package directory. This supplement excludes raw trace excerpts and code.
