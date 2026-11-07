# WAV descriptor limit writer boundary

## Bound selected factory path

On the inspected selector-3 Factory-open path, the selected caller allocates an AdmFileStorageDesc and calls its constructor with forwarded dynamic path/record arguments plus constants 0 and -2. The descriptor base initializer sets Desc+0x68 to zero. The constructor does not use the path or record arguments as that field's value.

The Factory-open method forwards the dynamic path and fields at record+0x10/+0x18 to the descriptor virtual method at +0x88. In the bound target body, that method has no explicit store to the same instance's Desc+0x68. Thus this selected construction/open path contains no observed nonzero write to the limit field.

## Remaining edge

The typed source of any later nonzero Desc+0x68 value remains UNBOUND. Another direct constructor caller exists but was not followed; this package makes no claim about that path or global absence. It reuses the previously washed initial range and reader contract without re-analyzing the +0xd8 reader.

The user-supplied V2 parent manifest file was unavailable in this checkout and is recorded only as a supplied hash. The upstream seal and checker passed normally and under python -O. No DEE run, runtime capture, media/PCM/payload/key/license read, or output occurred. Raw Ghidra traces are excluded from this supplement.
