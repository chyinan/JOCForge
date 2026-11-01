# Callback, source, dispatch, cache-origin, and meter clock crosswalk

## Scope

This supplement composes the approved GTC, codec-clock, frontend-normal, and meter-drain contracts into distinct clock domains for the selected primary example. It is contract/model evidence only: it does not inspect implementation source, attempt-07 logs, or native callback values, and it does not make the whole chain ready.

## Clock domains

| Domain | Rule | Scope |
|---|---|---|
| Main callback `g` | Zero-based, total `Q+4`; two priming callbacks precede source processing. Four cached main callbacks follow the `Q` normal callbacks. | Main callback owners advance once per successful parent commit. |
| Codec source `s` | Exists only for `g >= 2`; `s = g - 2`; total `Q+2`. | Source-stage owner clocks advance in source units. |
| JOC dispatch `d` | Exists when a source exists; `d = s + 1 = g - 1`; total `Q+2`. | JOC owner clock is distinct from main and source ordinals. |
| Cached payload origin | For each of four main cached callbacks, reuse the final normal snapshot at origin `Q-1`; current `g`, `s`, and `d` still advance. | Snapshot provenance, not the current callback/source/dispatch clock. Provider-embedded cache index meaning is unspecified. |
| Meter callback `m` | Independent range `0..Q+2`, consisting of `Q` normal callbacks and three fresh-zero drains; total `Q+3`. | Meter drain clock is not a main callback, codec source, or JOC dispatch. |
| `phase_generation` | Increment once for each successful top-level main callback commit; one separate METERING→READY commit after the full meter prepass. | Do not invent a per-meter-drain global generation delta. |

For the primary example only, `N=6,522,906`, `H=1,536`, and `Q=ceil(N/H)=4,247`. The last normal callback reads 1,050 real samples and pads 486 samples. These values are not constants for every input.

## Explicitly open

- F01E/F03W/F04 provider-embedded index semantics for cached candidates are not specified by the approved contracts.
- The F01E internal `normal_callbacks` transition during each main cached callback is not specified.
- A per-drain global `phase_generation` delta is not specified; only the one post-prepass parent commit is.
- Attempt-07's actual expected/actual mismatch values were not captured and are not reconstructed here.
