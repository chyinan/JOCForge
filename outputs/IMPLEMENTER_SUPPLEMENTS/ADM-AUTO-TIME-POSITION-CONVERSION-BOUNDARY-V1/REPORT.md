# ADM automatic time-position conversion boundary

## Wash disposition

The upstream sealed Ghidra receipt was independently checked. It closes useful conditional conversion and validation rules but not how `auto` obtains an effective offset or how the resulting value reaches the same reader's seek position.

## Static rules available for implementation

- `PrivContext::setRange` reads the offset map value and passes it through `std::stod`; the literal string `auto` is not accepted at this consumer. Any auto resolution must occur earlier, and its source remains `UNBOUND`.
- In the `embedded_timecode` branch, effective offset seconds are multiplied by the input sample rate (the inspected domain is 48 or 96 kHz) and converted with `CVTTSD2SI`, truncating toward zero to a signed integer sample position.
- In the `file_position` branch, sample-rate multiplication is skipped; this consumer treats the effective offset as already in the file-position/sample domain. The inspected sample-6 metadata selects `file_position`.
- FFOA uses `-1.0` as a sentinel. A statically observed guard for `-1.0 < FFOA < offset` logs invalid FFOA and writes the sentinel. Another warning says FFOA is discarded when it precedes master start, but the replacement value is not established.

## MVP boundary and remaining joins

The package explicitly reports that default `file_position` MVP is not blocked by this missing auto-source edge. Effective offset/FFOA values, same-reader seek sample, and leading-silence decision were not captured. Do not invent the upstream auto field or the post-discard FFOA value. No DEE run, PCM, payload, key, or license was accessed.
