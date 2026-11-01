# PCM24 intermediate to F32 consumer boundary

## Bounded finding

The hash-bound static trace follows one conditional PCM24 reader branch through its direct read wrappers and cursor/event handling. No numeric S16-to-F32 conversion was found within that traced direct-consumer slice. A separate four-byte float branch is not called from the traced three-byte branch.

A downstream DDP row-copy operation copies four-byte row data but is only a copy boundary. It does not establish a numerical conversion or typed identity from the conditional S16 intermediate.

## Wash disposition

No production PCM24-to-F32 rule is approved by this evidence. Do not infer scale, precision, rounding, or channel identity from row width, row count, or a copy operation. Continue the approved PCM24 input contract. The remaining minimum closure is one same-channel/sample/generation source scalar matched to the first numeric F32 consumer, using the already authorized single native run after its gate permits it.

## Source package

R3 manifest SHA-256: ec431936159b2fdbaf1b39c42e30d1b6c553cdb79232ae2c1c0cb40c5aec621f
R3 seal SHA-256: 834de899e1546c56802d12fec82358138d413a4b686f4457ecc0ee3d4b428364
Parent checker passed in normal and optimized Python modes. This Wash package contains no source code, disassembly, binary, PCM payload, private key, or absolute input path.
