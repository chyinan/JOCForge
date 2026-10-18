# QMF analysis direct vectors v1

Status: `IMPLEMENTATION_ACCEPTANCE_VECTORS; WHOLE_RECEIVER_NOT_RUN`

Use the fixed QMF analysis contract only:

- input: current `F32[64]`, before history `F32[9][64]`, analysis prototype and rotation provider words;
- fold: exact F04Q.2 order and R32 after every multiply/add;
- rotation: interleaved complex layout, provider rotation words;
- FFT input: rev6 permutation followed by component-wise R32 division by 64 before the first butterfly;
- FFT: fixed 64-complex negative-exponent primitive;
- reorder: output `p` then `63-p`, with the specified imag sign-bit XOR only.

The provided literals contain fold, rotated, scaled bit-reversed input, FFT output, reorder output, and next history. They are expected assertions after fresh computation; never load them as state or FFT output.

Provider identity remains package-relative: QMF functional provider SHA `30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c`; QMF mathematical FFT provider SHA `e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431`.

This package is lane-neutral. Receiver lane routing, synthesis, fitting, J05, parent transaction, real ADM, and listening remain deferred.
