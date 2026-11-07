# LFE recurrence selector correction

## Corrected branch source

The upstream hash-bound Ghidra 12.2 receipt follows `FUN_14036d9e0`, whose short selector chooses zero-branch inline recurrence versus nonzero `FUN_14036d7c0`. The caller review corrects the prior LFE interpretation: in both inspected core dispatchers, `FUN_140343530` and `FUN_1403448d0`, the LFE-lane branch assigns literal zero to the recurrence selector before calling the LFE main/companion wrappers. Those inspected LFE lanes therefore select the inline recurrence path.

The descriptor U16 stride at `descriptor+0x0c`, copied to `state+0x1320`, is used earlier to gather the 256-F32 input block. The local variable used for gather is overwritten before the recurrence call. It must not be treated as the recurrence selector. The nonzero table branch remains a separate FBW per-lane path in these two dispatchers; this is not evidence that another caller or sample uses the same route.

## Algorithm facts and limits

The inline branch mutates a 256-F32 temporary in place using 64 groups of four values and two 128-F32 coefficient regions. The nonzero branch uses 32 iterations over two 128-F32 banks and two separate 64-word tables. Both functions show F32 input/output in the decompilation and no persistent state mutation within those functions.

Instruction-level FMA/rounding and MXCSR are not bound; coefficient contents are not included; sample-specific layout/generation and numeric parity are not captured. The earlier LFE correlation near 0.819 is not remeasured and has no proven cause. The new finding corrects the stride→recurrence-selector join for the two reviewed dispatchers only; it does not establish a product-wide branch rule or Rust parity.

## Provenance

Upstream evidence-manifest SHA-256: `af45b96787219eb31b9a62c64e5c1b62590d2824ac37132404cf339e3509893d`; package-seal SHA-256: `0efe3846f472ae11bf27b874ab69f4ec78fa79569a41551b1088e709534f656b`. Upstream checker passed normally and under `python -O`; Wash independently rehashed its sealed files and source identities. This supplement contains sanitized formulas/status only; no Ghidra scripts/logs, binary, PCM, encoded payload, key, or license.
