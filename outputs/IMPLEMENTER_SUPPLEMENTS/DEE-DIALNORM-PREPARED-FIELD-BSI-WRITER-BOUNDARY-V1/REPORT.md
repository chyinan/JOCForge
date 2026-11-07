# DDPi DialNorm frame-preparation to BSI-writer boundary

## Disposition

DDPI_DIALNORM_TO_FRAME_PREP_BOUND_STATIC; FRAME_PREP_TO_FINAL_BSI_WRITER_UNBOUND; BIT_OFFSET_UNBOUND; RUNTIME_NOT_RUN.

The sealed parent evidence maps DDPi parameter ID 0x12 to per-stream +0x9c and frame-preparation context +0x191f4. The new bounded scan found exactly two direct +0x191f4 accesses in dee_ddp_encoder.exe, both in FUN_140348000: a 16-bit store from AX at RVA 0x3483a4 and a comparison with sentinel 0x33 at RVA 0x348768. The nondefault path sets update flags +0x191f2 and +0x191e4.

FUN_140348000 is called by FUN_1403448d0; that call is reached from FUN_140317600. The same frame-update function later calls FUN_14033b5f0 at RVA 0x317d28, and that candidate stuffer calls FUN_14036ac00 at RVA 0x33b76b. No direct read of +0x191f4 or typed alias into those stuffer arguments was found in this slice.

## Independent Wash checks

Wash recomputed all eleven source-package seal entries and both parent seals. The current dee_ddp_encoder.exe hash matched 0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532. The source package reports normal and python -O checker PASS; Wash did not execute the checker.

No DEE rerun, decode, PCM/payload/key/license read, runtime capture, or implementation contact occurred.

## Implementation boundary

Keep the parameter path closed through frame preparation only. The next candidate call is FUN_140317600 -> FUN_14033b5f0; neither final BSI serialization nor bit offset is established. This does not claim a PCM-sample effect or explain the prior DI A/B PCM hash difference.
