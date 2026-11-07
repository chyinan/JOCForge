# Line-mode DRC +0xA0 first row-pack boundary

## Bound static path

The line-mode U32 at snapshot+0xA0 is inherited from the sealed writer receipt. The per-frame packer receives the same snapshot pointer and chooses a row/vector pack path using snapshot+0x9C. For the zero branch, the first selected-row call goes to FUN_180182850; for the nonzero branch, it goes to FUN_1801431f0. Both receive the selected row pointer derived from param_4*0x20 + *param_3. Neither call receives snapshot+0xA0 as a scalar or field address.

## First break

The selected row pointer's owner/alias relationship to snapshot+0xA0 remains UNBOUND. No codec/BSI typed input is reached in this one-hop slice. The +0xA0 value's entry into a codec/BSI record remains UNBOUND; the two branches and runtime selection remain separate/unobserved.

Wash independently verified the upstream manifest, all seven payload hashes, SHA256SUMS, and package seal. The upstream agent reported normal and python -O checks passing; no checker executable was included. Raw trace excerpts are excluded from this supplement. No DEE execution or media/payload/key/license read occurred.
