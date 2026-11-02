# DEE E-AC-3 exponent-code producer and U4/U7 writer join

## Bound producer-to-writer edge

Ghidra 12.2 static evidence binds `FUN_14032d580` (U7 code producer) to the immediately following `FUN_14036f8f0` syntax writer through the same `state + 0x140f0` parameter base at their callsites in `FUN_1403448d0`. Both address the same per-block/per-lane count and code-row pointer tables.

For `j = 5 * block + lane`:

- U7 group count is `N = i16(state + 0x141c8 + 2*j)`.
- Difference count is `D = i16(state + 0x1418c + 10*block + 2*lane)`.
- Code-row pointer is `*(u64 *)(state + 0x143b8 + 8*j)`.
- The row's initial exponent code at `code_row_pointer + 0` is preserved as the U4 seed.
- `FUN_14032d580` computes `D` biased differences as `next_exponent - previous_exponent + 2`, using the selected per-lane stride. It fills positive tail length `3*N-D` with `2`, packs each triple as `25*d0 + 5*d1 + d2`, and writes U7 codes in place beginning at row byte offset `+2`.
- `FUN_14036f8f0` writes one U4 from row offset `+0`, then `N` U7 codes from offset `+2` on the same cursor: `4 + 7*N` bits.

## Confidence and remaining producer fields

Confidence is high (`0.98`) for the typed producer/writer code-row join and count-field identity, and high (`0.96`) for the transform/packing formula. `N` is read by `FUN_14032d580`; the upstream writer of `N` is still unbound. `N = ceil(D/3)` is only a model inference if the upstream plan chooses the minimum zero-to-two tail fill. The U4 seed's upstream producer and physical ADM-row-to-internal-lane mapping also remain unbound.

The 217-bin example (216 differences → 72 U7 groups) is an input-only model vector, not a DEE or ADM runtime observation. This is a direct E-AC-3 core syntax contract and a focused candidate for the reported AU0 exponent parse error; it does not prove that it is the only cause.

## Recommended trial

Use the same-row dataflow to build the current fullband lane's U4/U7 writer input and verify the count/cursor before emitting AU0. Keep the distinct U7-count producer uncertainty visible; do not replace it with `ceil(D/3)` unless the model's exact minimal-padding condition is satisfied. Then check the new full-track output's AU0 with an independent parser.

## Exclusions

No DEE binary, source code, disassembly, decompilation, media, PCM, private key, or license material is included. The evidence is bound by image/function hashes and synthetic arithmetic vectors only.
