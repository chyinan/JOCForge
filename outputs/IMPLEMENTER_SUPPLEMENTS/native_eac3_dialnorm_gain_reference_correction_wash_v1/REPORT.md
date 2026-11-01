# Wash review: E-AC-3 DialNorm gain reference correction

This separate addendum corrects the earlier public playback citation and identifies the normal-path five-bit field as DialNorm. The record +8 field is BSID; the distinct record +0x20 field is DialNorm by header order. A same-owner U16 at owner-relative +0x34 feeds the +0x20 field on the traced static path. The owner +0x34 upstream writer and meaning remain UNBOUND.

The public FFmpeg E-AC-3 decoder path copies dialog_normalization into decoder state and derives level_gain when target-level normalization is enabled (eac3dec.c lines 305-313); the shared decode path applies level_gain during coefficient scaling (ac3dec.c lines 1265-1269). Thus the DialNorm field can affect playback level. This does not reveal the R141 DialNorm value or justify an implementation value.

R141 did not hit the DialNorm writer; runtime value and generation remain NOT_CAPTURED/UNBOUND. The 16 captured at record +8 is BSID and must not be reused as DialNorm. Output payload was not read; playback and full E2E are not verified.
