# AtmosReader config U16 producer boundary

The hash-bound Ghidra slice traces the selected descriptor caller through the reader's typed owner. The reader private object stores its config pointer at `+0x58`; its descriptor setup obtains that pointer, zero-extends the config U16 at `config+0x0A` into a U64 descriptor count at `descriptor+0x08`, and copies the same config pointer to `descriptor+0x28`.

The selected CompositeMaster open path writes the count U16 at `CompositeMaster+0x1A` and the channel-map pointer at `+0x20`, then returns `CompositeMaster+0x10` as the reader config. Thus the reader fields are config `+0x0A` and `+0x10`. The traced writer has no fixed 36 clamp/default; it writes the count selected by the ordinary/special formula path.

For `人是猫`, washed metadata gives 38 CHNA tracks (10 beds + 28 objects), so 38 is a practical profile candidate and the earlier reader→core path can carry it. The same-run reader U16 is not captured. Do not claim that the 4-second DEE run used 38 without a receipt. Preserve the dynamic count and row order; avoid hardcoding 36.

This capacity/count evidence does not by itself explain the AU-aligned PCM residual. Ghidra 12.2 static only; no BWF/PCM/payload/key access or DEE run occurred in this slice.
