# Mode-1 reader selector and layout-count owner edge

## Disposition

COMMON_OUTER_ARGUMENT_BASE_BOUND; READER_SELECTOR_AND_HOST_MODE_ARE_DISTINCT_FIELDS; PHYSICAL_ADM_N_AND_SOURCE_K_UNBOUND; RUNTIME_NOT_RUN.

The addendum binds two mode-1 fields to one caller argument base, but does not establish that the reader-variant selector chooses the open-config mode, nor that the reader-produced count is physical ADM N or source-descriptor K.

## Bound fields

In FUN_180175500, param_2 is the common base. FUN_1801764e0 reads the reader selector as U8 at base +0x140: zero selects AtmosReader and nonzero selects AtmosReaderCC.

FUN_180175500 also calls FUN_180177e50 with an input derived from param_2 +0x1B. The constructor FUN_180177eb0 copies a U32 from its input +0x54 to the new host object +0x54. Relative to the common base, this is a distinct U32 field at +0x12C. FUN_180178dc0 reads that host field and sets open_config[0] when it equals 1.

Therefore the fields share a base but are not the same field, and no direct dataflow from the +0x140 reader selector to the +0x12C host-mode field is shown. Their runtime values and the selected reader variant were not captured.

The prior mode-1 trace binds a reader-produced descriptor U16 at descriptor-pointer +0x0A to open_config[2], via record slot 5. That descriptor pointer has not been typed/identity-joined to physical ADM N or source K. Equal counts are not an ownership join.

## Independent Wash checks

Wash recomputed the source manifest and all nine listed files, the two local Ghidra trace hashes, the referenced mode-1 count-producer and source-K manifests/reports, and the current target DDP plugin DLL hash. The DLL matched 3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2. The source package makes no formal package-seal claim; its provenance status file is retained by hash. The source agent reports normal and python -O checker PASS; Wash did not execute that checker.

## Implementation boundary

This is a mode-1 input/count ownership boundary only. Keep the host mode, reader variant, descriptor count, physical ADM row count N, and source count K as distinct typed values until a concrete join is shown. Runtime remains NOT_RUN.
