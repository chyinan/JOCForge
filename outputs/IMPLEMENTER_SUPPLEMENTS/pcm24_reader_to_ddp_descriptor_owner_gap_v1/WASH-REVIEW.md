# PCM24 reader to DDP source-row owner gap

This Wash addendum verifies the hash-bound Ghidra static evidence for both ends of a remaining cross-image edge.

On the converter side, the selected context-7 reader entry reaches an adapter that accepts the caller's row-pointer table and creates a borrowed F32 row view. The view is adjusted by the sample offset using four bytes per F32 sample. The selected PCM24 normalization rule remains the separately washed `signed_pcm24 / 2^23` rule.

On the DDP side, the callback's second argument is forwarded as a descriptor. Descriptor offset 0 is used as a source-row table; offset 8 carries channel count; offset 16 carries frame count. The prior washed package closes the per-row copy into the DDP internal buffer.

No writer of descriptor offset 0 or outer DDP virtual callsite was located in the two scoped images. No typed store/direct argument or owner-generation receipt connects the converter's borrowed row view to that descriptor field. This remains UNBOUND/NO; endpoint presence and compatible shapes are not a join. The minimum next artifact is the host writer/adapter assigning descriptor offset 0 and invoking DDP with the same input owner and parent generation.
