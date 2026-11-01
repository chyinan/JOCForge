# Run151 host and DDP reader adapter boundary

This supplement carries only the bounded control-flow observations in the declared scope. It does not specify an audio transform or establish selected ADM ownership.

Ghidra 12.2 static mapping traces the host RunInput call into StageRunner, the stage callback into InputObject execution, then the inner interface dispatch to AtmosMezz Init and Run. Run151 observed those host stops and the DDP reader stops on one thread; the same AtmosMezz inner-interface token appeared at Init and Run.

Within the DDP measurement owner, a 336-byte local source record is initialized and passed to the AtmosReader conversion path. The record pointer is converted into a distinct local reader configuration. That configuration pointer is then forwarded unchanged through the reader, private-state, and composition-state constructors. The descriptor offset-zero store used the same token as the DDP row-table argument. No row content was captured.

Important input boundary: the runtime capture labels its input as the Run145 First Love ASCII alias and cites a preflight SHA, but says the input was not rehashed during this capture. Treat this as preflight context only; this run does not prove the selected ADM identity or generation.

The encoder-done marker and 32,964,608-byte pre-cleanup file size are context only. The output bitstream was not parsed, playback was not run, and output files were removed.
