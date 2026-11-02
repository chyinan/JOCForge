# DEE bit-writer to AU final-word boundary

The same writer state at `state+0x20648` is passed into field writing and
field-end processing. Each output unit initializes the current word pointer
and bit-position base from the selected output pointer and clears the cursor
and accumulator. Field-span bookkeeping computes the bit interval but does
not flush pending bits.

After the field-end helper succeeds, the unit finalizer checks the cursor. If
nonzero, it writes the MSB-aligned accumulator as one full U16 word; unused
low bits provide zero padding to the next 16-bit boundary. The flush does not
advance the pointer or reset the cursor; the next output-unit iteration resets
the context. This is an actionable final-word rule for the local writer.

The first per-unit 56-byte output record is medium-confidence as a complete AU
descriptor. A later consumer reads a pointer at `+0xb90` and byte count at
`+0xb9a`, but no alias connects that buffer to the pointer selected by
`uVar28`. Thus the writer's residual-word handling is closed while complete
AU buffer identity and file parseability remain unproven by this slice.

Use this contract to check the model's bit-writer finalization. It does not
itself establish that the current `.ec3` output parses or plays. Static Ghidra
evidence only; no DLL runtime, media, PCM, payload, key, or license access.
