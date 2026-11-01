# F06 custom BSI metadata versus standard EC3 header path

## Distinct producer paths

Ghidra traces the active BSI owner's signed U16 at `+0x34` into a custom BSI serializer. Its output buffer and length flow into a metadata item of type `0x0B`, then a two-entry metadata container consumed by frame composition.

The standard EC3 syncframe header's `dialnorm` is a separate five-bit field at header bit offset 45. The visible encoder frontend passes a separate per-frame `dialnorm` parameter through the encoder request. No typed transfer from owner `+0x34` to that parameter or U5 field was found in this trace. These structures/producers converge in the final AU, but the evidence does not provide a numeric U16-to-U5 mapping.

## Existing generation evidence

The prior same-generation package binds one typed BSI owner through serialization/composition to a nonempty 4096-byte FileWriter call at frame index 3481; a bounded seven-byte header probe at that offset reported U5=17. It did not reread U16 during that full run. The separate owner-default package records zero initialization for the typed owner path, but alternate writers remain unexhausted. Do not combine the short-run U16=0 with the full-run U5=17 as a same-field or same-generation numeric pair.

## Disposition

Reference only; `implementation_ready=false`. The custom metadata path is structurally distinct from the standard header U5 path. Keep U16-to-U5 transfer, backend U5 conversion, clamp, rounding, and valid range `UNBOUND/NOT_PROVEN`. No audio algorithm rule is added.

No new DEE run occurred. The incoming static trace read no PCM or codec payload and accessed no license/key content. The only prior output observation referenced here was the seven-byte bounded header probe described in its own package.
