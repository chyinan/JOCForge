# F09LB selected LFE gate and mantissa boundary

This direct slice implements only the selected-profile LFE gate source, the shared finite-F32 PRIMARY/LFE clipping and raw-exponent scan, and a staged seven-word LFE coefficient-row owner.

The selected profile gate accepts only 48 kHz, 1,024 kbps, acmod 7, six blocks, and AHT disabled; it returns gate zero. PRIMARY rows accept 0..=253 words, while LFE MAIN requires exactly seven. Non-finite words reject before any staged row exists. Strict comparisons clip only values below -1 or above +1; equality, signed zero, and finite subnormals preserve the input word. Exponents are derived from the clipped word by the supplied unsigned doubling and exponent-bit rule.

The state machine keeps the committed clipped row and raw exponents together. A downstream failure preserves the old bank, commits nothing, and retires the machine; subsequent staging is rejected. Reset restores gate zero, an empty bank, and an active machine.

F10.4 numerical LFE normalization, the nonzero-gate zero-factor path, D06-N provider binding, full F09/F10 plan production, AHT-enabled profiles, and source/AU or parent transactions are deliberately excluded. The package's generic normalizer vectors are retained as source material but are not claimed as implemented by this slice.
