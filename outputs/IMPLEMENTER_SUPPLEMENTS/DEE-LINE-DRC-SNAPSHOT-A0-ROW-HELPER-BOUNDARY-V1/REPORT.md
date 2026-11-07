# Line DRC snapshot +0xA0 row-helper boundary

The selected snapshot branch passes a selected row pointer and local vector arguments to its first row helper. It does not pass the snapshot +0xA0 U32 or its address as an explicit argument. The selected row pointer has no typed owner or alias join to that U32.

The snapshot pointer reaches the per-frame packer, but the DRC value's lineage breaks at the packer-to-row-helper call boundary. Indirect helper transfer, codec/BSI input, runtime branch, and audio effect remain UNBOUND or unobserved. The package follows one helper edge only and makes no global absence claim.
