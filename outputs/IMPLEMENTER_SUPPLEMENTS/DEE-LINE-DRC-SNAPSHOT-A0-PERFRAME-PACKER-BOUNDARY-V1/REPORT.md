# Line DRC snapshot +0xA0 per-frame packer boundary

The parent receipt binds a Line DRC profile-tree result as a U32 written to snapshot byte +0xA0. In the bounded mode-1 continuation, the same snapshot base reaches a validator and then a per-frame packer. The validator checks vector lengths and string presence; it has no direct +0xA0 read. The per-frame packer receives the snapshot base but likewise has no direct +0xA0 read or copy in the inspected edge. An earlier field at +0x130 is distinct from the DRC U32.

The record pointer reaches per-frame validation/packing, while the +0xA0 value is not typed-joined to that pack or a codec/BSI input. Indirect row/helper transfer and any decoded audio effect remain UNBOUND or undetermined. This is a bounded field-specific continuation, with no global absence claim.
