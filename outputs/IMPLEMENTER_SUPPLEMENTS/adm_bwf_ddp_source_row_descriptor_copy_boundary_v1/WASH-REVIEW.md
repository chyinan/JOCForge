# DDP source-row descriptor and copy boundary

This Wash addendum independently checks the R3 source package manifest, seal, five artifacts, and all nine hash-bound Ghidra evidence files. Its checker passes in normal Python and `python -O`.

Static evidence closes the DDP-side mechanism: a callback's second argument is forwarded into the DDP processor as a descriptor. By use, descriptor offset 0 is the source-row pointer table, offset 8 is channel count, and offset 16 is frame count. Under the full-buffer capacity condition, source row `i` is copied to internal row `i` for `frame_count * 4` bytes.

The included input-only vector uses synthetic Float32 bit patterns to illustrate that copy. It is explicitly NOT_RUN and is not PCM or an encoder capture.

The selected converter's borrowed context-7 PCM24-to-F32 row table is still not connected to descriptor offset 0 for the same input owner and parent generation. The sample-6 parent run did not hit the selected reader branch. This supplement closes only the DDP descriptor-forwarding and row-copy contract; it does not establish the source-row producer, audio-algorithm behavior, or output conformance.
