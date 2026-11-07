# Composite output-format channel count boundary

Within the same composite-master owner, both layout paths write a 16-bit count to owner+0x1A. The returned metadata base is owner+0x10, so this is metadata+0xA, which the reused Reader producer reads for its channel count.

When layout flag owner+0x100 is zero, the count comes directly from the format-shape interface at owner+0x48, virtual slot +0x80. That same interface's +0x88 supplies the six-byte mapping entries copied to the output map. When the flag is nonzero, the function walks the per-source interface vector, reads each source's count and entries through slots +0x80/+0x88, counts entries whose byte at entry+0x2 is 0xFF, takes the maximum per-source count, and adds ten.

Both stores are behind a callback gate. The function then allocates six bytes per output-map entry; allocation failure returns before handing out metadata, and a post-format callback can also prevent pointer handoff. These are static control-flow facts. The runtime branch remains NOT_CAPTURED, and neither branch is joined to the physical CHNA row count for a real input. No general N range or 36/38 branch selection is established by this package.
