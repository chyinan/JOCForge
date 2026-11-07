# E-AC-3 JOC rate writer-state consumer boundary

Within the same FUN_1801c0770 callback, a U32 kbps field is multiplied by 1000 into a local record. After validation succeeds, FUN_1801f8b80 copies that input qword into writer_receiver+0x20. The qword's low 32 bits carry the scaled rate; the high 32 bits come from an adjacent local field.

The first subsequent reader is FUN_180236590 in the same callback. It uses the low 32-bit rate with writer-state fields to select a table row and, when a nested output pointer is non-null, writes row data to nested offsets +0x4, +0x8, and +0x10. FUN_180231ce0 then copies the nested +0x10 U32 into +0x4 of an object reached through writer_receiver+0x38.

This binds a typed transfer within one writer receiver through a generic U32 field store. The destination type is unknown; no codec, BSI, frame-size or bitstream meaning is established. The earlier slot2 rate code/setup to the slot4 rate handle+0x18370 still lacks a same-runtime-handle/owner/generation join, so a specific public rate selection such as 1024 kbps is not proven to reach this path. No DEE run or payload access occurred.
