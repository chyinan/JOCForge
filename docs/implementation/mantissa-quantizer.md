# F12S.5 mantissa quantizer

This stateless slice consumes a finite clipped coefficient word, a decoded I16 exponent, and a BAP value. It evaluates `coefficient × 2^exponent` in F64, converts once to F32, then applies the selected quantizer with separate R32 arithmetic steps. The exponent remains an upstream F09/F10 value; this slice does not derive or revalidate its producer range.

BAP 0 returns code zero. BAP 1/2/4 use clamped truncation; BAP 3/5 reject their reserved all-ones sentinel; BAP 6–15 use the specified width-dependent offset, separate R32 additions and multiplications, ties-to-even rounding, arithmetic shift, and signed-16 clamp. Group formation and MSB-first block packing are implemented separately in the F12S.6 block packer.

The Wash package is `CODEC-QUANTIZER-MANTISSA-PRODUCTION-CONTRACT-V1`, manifest SHA-256 `fd819c3ec27e7bade55efda5e648792c9c33892a9be7a36bbfce89231b9e961f`. Its vectors contain 45 input recipes and intentionally include no expected quantizer codes. Local tests verify accepted input domains, sentinel failures, and selected formula/rounding examples; they are not native or full-frame vectors.

F09/F10 production, F12A BAP generation, full frame budget/close/CRC, retry, source clocks, parent commit, native replay, and product acceptance remain open.
