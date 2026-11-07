# Sampler float/state return boundary

## Disposition

STATIC_RETURN_TO_SAMPLER_STATE_BOUND; PCM_AUDIO_CONSUMER_UNBOUND; CONDITIONAL_PLUS80_GATE_PRESERVED; RUNTIME_NOT_RUN.

The new Ghidra 12.2 slice follows the conditional return of FUN_1801caa90 to its first typed state owner. It does not identify a PCM or AtmosProcessor audio-block consumer, and it does not explain the earlier observed PCM-hash difference.

## Bound edge

FUN_1801b6fa0 captures the 64-bit return and 32-bit companion from FUN_1801caa90 into local_78/local_70, then writes them to the same nested receiver at +0x8c and +0x94. The same wrapper also copies a state snapshot through its param_2 output. At the examined FUN_1801ad9d0 callsite, RDX points to caller-local local_d8.

Neither the nested state fields nor local_d8 are shown as a PCM/audio-block pointer. The typed edge from those owners to an audio sample buffer remains UNBOUND. The +0x80 condition remains conditional; no runtime value was used to decide whether it executes. This does not repeat the separate row-output-to-FUN_1801acd10 metadata result.

## Independent Wash checks

Wash recomputed all four source-package payload hashes and lengths, the five SHA256SUMS entries, the manifest and seal bindings, all 17 source-manifest entries, both parent manifest hashes, and the target DLL hash. The target DLL matched 3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2. The source agent reports normal and python -O checker PASS; Wash did not execute that checker.

No DEE run, runtime capture, PCM/media/AU/EC3 read, key/license read, or implementation contact is claimed.

## Implementation boundary

This receipt only prevents treating the sampler-state recurrence as PCM evidence. It supplies no DSP formula and does not close the PCM mismatch.
