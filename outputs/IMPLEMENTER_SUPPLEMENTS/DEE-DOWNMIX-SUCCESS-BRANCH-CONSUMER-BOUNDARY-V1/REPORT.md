# DEE downmix parser success-branch consumer boundary

The focused Ghidra 12.2 scan starts from the parser's caller-owned output slots `+0x1ac` (`lorosurmixlev`) and `+0x1b0` (`dmixmod`). It did not bind a first non-parser reader of the same object. Same-offset candidate reads were rejected because their object types did not match: one treats `+0x1ac` as a byte count and `+0x1b0` as a pointer; another treats `+0x1a8/+0x1b0` as begin/end pointers for an 8-byte-element container.

Thus the `dmixmod`/level-to-audio-matrix or 5.1 frame-input edge remains `UNBOUND`. This does not establish that the fields are ignored, and it is not an audio algorithm result. It neither explains nor clears the current PCM mismatch. Do not spend the current window-table trial on guessed downmix defaults; resume this edge only if an exact typed consumer is found.

Ghidra 12.2 static only. No DEE run, PCM/payload/key/license read, or output comparison occurred. No raw trace or Ghidra script is included.
