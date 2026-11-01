# PCM24 reader to DDP host dispatch boundary

This Wash addendum rechecked the bounded Ghidra package and summarizes the cross-image edge. The selected ADM reader exposes a borrowed F32 row table; the DDP consumer treats descriptor offset 0 as a row-pointer table and copies rows by ordinal. The host reaches the selected stage through virtual dispatch, but the audited host path has no typed store from the reader table to descriptor offset 0.

The reader-to-descriptor owner/generation join remains UNBOUND. Static vtable references do not identify a live invocation or concrete adapter. The exact next edge is the runtime target of the host stage-sequence dispatch, followed by the adapter store into descriptor offset 0 in the same process and parent generation.

No DEE job or runtime capture was performed. The prelaunch XML had no key/auth/token/license-named fields, but DEE launch depends on an external license file whose contents were not read. This package contains no source, decompilation, binary, PCM, payload, or license material.
