# I0/D0 access-unit commit boundary

## Bounded native observations

The reviewed Ghidra 12.2 package binds two separate paths in the selected candidate image:

1. A DDP per-unit path submits unit records in increasing ordinal order; `FUN_180077ac0` receives a source span and a separate caller buffer; `FUN_1800e01e0` writes completed bytes to that in-memory buffer.
2. `FUN_180175c80` obtains a span from a provider, computes its byte length, passes it to `FUN_180189c80`, and reaches `fwrite` for nonzero spans. The provider selector can return an `EmpireEnc` or `DDPJocEnc` provider, but the selected provider for the target is unbound.

No typed pointer, count, source-generation, stream-role, or AU-identity edge joins the per-unit callback span to the FileWriter span. The static observations do not establish that either unit is I0 or D0, or that two writes form one same-generation AU.

## Target contract and implementation seam

The user target requires one I0 followed by one D0 in a single access unit. The target profile and new I0 frame candidate define structure requirements; the native path above does not prove a durable paired commit or rollback behavior. A pair-level in-memory candidate can be developed from same-generation typed inputs. File publication should remain behind an explicit model-owned transaction gate until its source/commit join is established.

`production_allowed=false`. This contract contains no algorithm, no media, no raw frame, no PCM, no compressed payload, and no license/key material.
