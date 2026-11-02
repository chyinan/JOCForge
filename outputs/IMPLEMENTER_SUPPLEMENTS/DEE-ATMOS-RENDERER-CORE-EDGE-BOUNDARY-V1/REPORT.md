# Atmos renderer to DDP core boundary

## Closed route fact

In the hash-bound Atmos plugin image, `inner_context+0x80` is a constructed `SinglethreadSymphonyRenderer` object with vtable `0x1807a7808`. Its `+0x08` slot is the per-frame callback `FUN_180423e90`; `+0x20` is a separate getter. The small setup value stored at `inner_context+0` and the separate setup table `PTR_LAB_1809f6630` are not this callback. The callback delegates through `FUN_180423ce0` and dynamic renderer-child vtables.

## Remaining LFE/core boundary

The callback's third argument is the selected descriptor array, but the decompiled callback body does not read it. Child dispatch is configuration-dependent and no child target is statically bound to the selected DDP core image. The row-binding helper stores pointer/extent metadata; actual sample consumption and any live 256-F32 extent are not captured. The earlier standalone DDP core candidate remains unjoined to this renderer instance and generation.

This narrows the route ambiguity, but does not establish the selected LFE transform or explain the audio difference. Next step is a single run-scoped metadata/pointer-token trace to the actual child consumer, repeated on another authorized ADM if needed. Do not inspect buffer contents or infer core ownership from matching shapes.

## Limits

Static Ghidra 12.2 evidence only. No DEE execution, PCM/media/payload/key/license access, audio comparison, or implementation change occurred. Selected core transform remains `UNBOUND` and parity remains `NOT_RUN`.
