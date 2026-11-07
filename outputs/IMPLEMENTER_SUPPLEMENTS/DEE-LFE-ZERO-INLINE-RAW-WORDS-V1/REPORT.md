# DEE LFE zero-inline recurrence coefficient vectors

This package contains the 128 binary32 raw words for each of the two coefficient tables used by the zero-inline recurrence, copied from the hash-bound Ghidra memory export. The words remain bit patterns in ascending address order; no float reformatting is applied.

| Table | Base | Count | Bytes | Raw-byte SHA-256 |
|---|---|---:|---:|---|
| zero-branch table A | `0x140628c20` | 128 | 512 | `bbd2bd460b0a750ccb45f959561a9a7e2ed2f8c6fd9391fb21abc9216b08fa41` |
| zero-branch table B | `0x140628e20` | 128 | 512 | `e4cbb855ed455887b387cdb188bf8bab41976c73648c1b022dd18156214bde50` |

The corrected DEE branch receipt says the two inspected LFE dispatcher paths select the zero-inline recurrence. The current F08 candidate package lacks corresponding 128-word A/B candidate values, so direct table comparison remains `NOT_RUN`; this package does not claim match/mismatch, numeric parity, sample-specific branch selection, or the cause of the reported correlation.

## Provenance and limits

Source package `dee_lfe_zero_inline_raw_words_wash_v1`: evidence manifest SHA-256 `52f001fc2a3215e103899db8358eeb5836b258254acbbd7fc13202e46c3e1eb1`, seal SHA-256 `1a708cbc3195a6a60369e6680ffc25b83cc8f70676537128b7a4a759cc242493`, raw-word payload SHA-256 `2468568363d487361c8359e50362ebc5502414e6e3924f2486cb31a71c523f54`. Ghidra 12.2 read-only/no-analysis; no DEE run, PCM/payload/key/license access, or implementation-source read.
