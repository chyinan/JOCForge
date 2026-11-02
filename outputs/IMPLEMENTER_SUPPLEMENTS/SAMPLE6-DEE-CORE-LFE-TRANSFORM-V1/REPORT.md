# Human Cat DEE core LFE transform path

## Core input and layout routing

The hash-bound E-AC-3 core input interface receives six channel descriptors. Each descriptor carries a sample pointer and a U16 sample stride. The core copies these into pointer and stride tables. A layout selector chooses the internal pointer for each lane; lanes at or above the full-bandwidth-channel count take the dedicated LFE main/companion branch.

The matched run's selector value, selected layout row, and ADM row-3-to-core-slot mapping were not captured. This package therefore identifies the core algorithm and its branch condition, but does not claim that the user's confirmed low-frequency source row 3 reached a particular internal LFE lane in this run.

## First sample-valued operation

For each internal lane, the core copies 256 F32 words from the selected input pointer using that channel's stride. The first sample-valued operation is a 64-step symmetric F32 pre-rotation. For iteration `i`, coefficient-table values `C[i]`, `S[i]`, `C'[i]`, and `S'[i]` form these pair results:

```text
even pair:
  x[2i]       * S[i]  + x[255 + 2i] * C[i]
  x[255 + 2i] * S[i]  - C[i] * x[2i]

odd pair:
  x[2i + 1]   * S'[i] + x[254 + 2i] * C'[i]
  x[254 + 2i] * S'[i] - C'[i] * x[2i + 1]
```

The results update transform work/history rows. The call refreshes words 0..255 but reads through relative F32 index 381 (382 words). The owner, capacity, and previous-generation source for words 256..381 remain unknown; this history edge can change the output and must be modeled before comparing exact values.

## LFE continuation

After the shared pre-rotation, the LFE branch builds separate main and companion banks. A stride-dependent stage either uses an alternate helper for nonzero stride or an inline recurrence for zero stride; a following stage forms a 16-F32 precursor. Each bank emits seven F32 values. The package records the main/companion formulas in EVIDENCE.json, but does not assign a measured sample output.

## Scope

This is static Ghidra evidence, not a matched-runtime numeric comparison. The input-only vector is symbolic and not run. The prior 4096-point spatial-analysis weight is a different branch and is not used as an LFE amplitude gain here. No PCM, payload, private key, license material, source code, or disassembly is included.
