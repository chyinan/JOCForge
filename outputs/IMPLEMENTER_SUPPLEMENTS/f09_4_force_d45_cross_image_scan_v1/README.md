# Force-D45 source scan: bounded cross-image result

The R5 package reports a Ghidra 12.2 scan across seven hash-bound installed images. It found no direct later nonzero store to owner+0x135c0 in those images; the single-pass, standard DDP and transcode images contain the same constructor-clear/read-copy pattern. The Atmos, converter, COD and DACTLib scans found no relevant data-store references in their bounded search.

The reference observation did not capture owner state: the DEE job was attempted once with NUL output/log sinks, WinDbg breakpoints remained unresolved, and the target exited before capture. No retry was run.

This supports the first-generation zero initialization but does not prove permanent zero. Indirect writes and writers outside the seven scanned images remain possible. Implementation may use the cold-start zero and traced copy path only within that scope; keep later update behavior open.
