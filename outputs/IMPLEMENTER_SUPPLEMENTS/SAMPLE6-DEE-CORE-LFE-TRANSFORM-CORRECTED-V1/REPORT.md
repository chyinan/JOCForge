# Corrected Human Cat DEE core LFE transform path

## Corrected input and history ownership

The previous R4 report incorrectly combined accesses from separate transform parameters and said the input scratch was read through F32 index 381. The corrected Ghidra trace shows that input `param_1` reads only F32 indices 0..255. Its symmetric partner indices are `255-2i` and `254-2i`, not `255+2i` and `254+2i`.

The input scratch pointer is held at core state `+0x262e8`. The heap path requests `0x420` bytes from the encoder arena, which reserves two `0x400`-byte units and aligns the usable pointer to 32 bytes. The constructor/cleanup fallback points to aligned inline storage at `state+0x25ee0`, with exactly `0x400` bytes for 256 F32 words. There is no input scratch tail beyond word 255.

The transform's other pointers are distinct: persistent per-slot state is `state+0x266f0 + selected_map_slot*0x400`, and the temporary is separately allocated at `*(state+0x296f0+0x420)`. Thus the earlier “scratch words 256..381 history source” claim is withdrawn. The persistent-history owner is now identified structurally; its matched map slot and runtime contents remain unbound.

## Core LFE path

The core receives six channel descriptors (pointer plus U16 sample stride). A layout selector chooses the descriptor for each internal lane; lanes at or above the full-bandwidth count enter the LFE main/companion branch. The matched job XML binds `encode_to_atmos_ddp`, 1,024 kbps, and the input filename, but does not itself specify sample rate/channel layout or the core mode/layout selector. The matched selector, selected row, and ADM row-3-to-core-LFE-lane mapping remain unbound.

After the 256-word strided copy, the first sample-valued transform is a 64-step symmetric F32 pre-rotation. For iteration `i`, it computes:

```text
even pair: x[2i]*S[i] + x[255-2i]*C[i]
           x[255-2i]*S[i] - C[i]*x[2i]
odd pair:  x[2i+1]*S'[i] + x[254-2i]*C'[i]
           x[254-2i]*S'[i] - C'[i]*x[2i+1]
```

The LFE continuation produces separate main and companion banks of seven F32 values per block. Its layout/stride and matched-row mapping remain unbound; this is a static algorithm path, not a matched audio result.

## Current window coefficients and input-only vector

The old 12 one-ULP window-table mismatch came from a historical Rust provider. Wash directly rehashed the current Rust parameter file and confirmed its 256 window words match DEE; the old mismatch is superseded, not a current LFE difference.

The row-3 two-tone vector in `INPUT-ONLY-VECTOR.json` remains symbolic and `NOT_RUN`. No expected values or PCM causality are asserted. No PCM, encoded payload, private key, license material, native source code, or disassembly is included.
