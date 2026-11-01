# F06 BSI owner to output-frame writer observation

One full DEE reference run on the authorized Human Cat ADM used an ASCII alias with the same SHA-256. The run exited 0 and produced a task-local EC3 containing 4,247 complete 4096-byte frames. On the same observed processing generation, the BSI serializer owner and the FileWriter frame input were linked by the run trace. One writer call extended the file by exactly 4096 bytes at frame index 3481.

A separate seven-byte header-only parse at frame index 3481 observed an independent 48 kHz, six-block, 4096-byte E-AC-3 syncframe with DialNorm U5=17 (-17 dB). No frame body or other EC3 bytes were read. The output file was not retained or content-hashed. This is a DEE reference observation, not a JOCForge output or full codec conformance result.

The current-run BSI owner U16 was not read. A separate earlier measurement callback observed U16 zero on its own generation, but this package does not join that value to the current frame; the numeric U16-to-DialNorm relation remains NOT_BOUND. Owner construction default and alternate writers also remain open.
