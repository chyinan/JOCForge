# DEE source_weight native sample confirmation

Disposition: NATIVE_HELPER_SAMPLE_OBSERVED; NATURAL_CALLBACK_AND_FULL_ENCODING_NOT_RUN. This is a bounded target-DLL observation and does not add a product-wide native-bit-parity gate.

## Source binding

Source package sourceweight_native_callpath_confirmation manifest SHA-256: 4fa5a08ffe2dd7bde89890c780ad9e40af0194447f46a6e8de8b0f6f96a4f655.
Source static-call-path payload SHA-256: 78f9878e7f6e245eaf37e046d3ca65b706c65b87a28b20ddb4337fd54e08bca5.
Source native-observation payload SHA-256: 067f571120087ef31cfbf64f39af46284d1342b7adc9ba48fdd65069513ca2ad.
Target DLL SHA-256: 3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2.
Approved mathematical-model package manifest: ab3e1495898bd1cbf36a8c449fabcccfb64ac747c7b2a1955665362ceceaa0ab.
Wash verified all source manifest payload hashes and ran the evidence checker in normal and optimized Python. No source, decompilation, address, raw PCM, EC3, key, or license material is copied.

## Static path reported for the exact DLL

The source package reports a 62-instruction source_weight loop and a 744-instruction local powf callee byte-matched to the target DLL. source_weight starts a positive-zero F32 accumulator, loads each band feature, calls the in-image powf, performs scalar F32 add, advances by source stride, and stores one weight per source. Exponent word: 0x3e4ccccd. The PE has no imported pow/powf symbol. The local powf has an ISA guard selecting an FMA3-labeled branch with five FMA instructions when the observed dispatch value is nonzero. Upstream library identity is unknown. The operation-list/byte-match claim is attributed to the source package; Wash did not reproduce disassembly.

## Native helper vectors

Using the hash-bound target DLL in the recovered DEE host, three direct helper calls each processed one synthetic source and invoked local powf 24 times. The natural host callback into this helper was not observed.

- 24 copies of 0x3d0ac7c7: native power 0x3f02163f and source_weight 0x4143215f; approved mathematical model gives 0x3f021640 and 0x41432160, a one-ULP difference for this vector.
- [1.0, 2^-120, 2^-120]: native total 0x3f800000, matching the required band-order F32 left fold; regrouping small terms gives 0x3f800001.
- Maximum finite F32 and minimum positive subnormal: powers 0x4c420303 and 0x30930889; native total 0x4c420303.

These are synthetic input-only results, not values read from the authorized WAV. They show one sample-specific approximation difference and confirm the tested fold order. They do not establish all-domain parity or natural callback behavior.

## Run limits

The authorized 38-channel BWF and target DLL identity were bound. The observer ran inside the DEE host after plugin registration recovery and directly called the helper. Natural source_weight callback, invalid-domain native inputs, matrix selection, U04, full encoding, output, playback, and product acceptance remain NOT_RUN. Final output bytes were zero. The implementation should follow the approved mathematical F03C/F03U model and its input-only vectors; the observed one-ULP target result is characterization evidence only, not an acceptance target.
