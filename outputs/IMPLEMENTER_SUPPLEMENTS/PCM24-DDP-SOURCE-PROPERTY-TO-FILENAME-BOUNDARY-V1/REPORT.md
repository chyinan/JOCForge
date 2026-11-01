# PCM24 DDP source property and filename boundary

## Static finding

For the hash-identified `dee_audio_filter_ddp.dll` image `2fd503…39ecd72`, Ghidra traces `FUN_1801ed430` handling the property key `dee_audio_filter_source` into parser `FUN_18020d820` at callsite `0x1801ee8e7`, then metadata lookup `FUN_18020c490`. This is a static property-to-parser edge only; no runtime setter key/value was captured.

A separate function, `FUN_18023f700`, reads a `filename` key from a `std::map`, converts the associated value, and copies it to an output string. Ghidra reports no direct code caller for this function. The evidence does not connect the source-property parser to this filename reader or to the host R138 `audio_input` owner.

## Disposition

Reference only; `implementation_ready=false`. Keep property-to-filename dataflow, host input-object identity, owner, and generation `UNBOUND`. The linked capture has `setter_record=NOT_CAPTURED`, DEE audio consumption `UNKNOWN`, bitstream/playback `NOT_RUN`. No parser alias, filename rule, or ADM read behavior is promoted.

The source package checker passed normally and under `python -O`. It records no PCM, payload, audio output, or license/key content read.
