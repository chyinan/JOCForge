# Public Enable Atmos 7.1 patcher boundary

Reference-only supplement from the public repository commit `459b23b6eef609edc831642a3084e4d0af7e1e32`. The code is a version-locked binary patch utility: it checks the whole input binary hash and expected byte patterns, patches internal layout-selector immediates, and verifies the resulting patched-binary hash. It does not encode audio or parse, modify, or validate EC-3/AU0 payloads.

The README claims that DEE 5.2.1 mode 21 restores a flat 7.1 path and reports dependent chanmap `0x1A00). The repository contains no encoded sample or run receipt to independently verify that real-encode claim. Its Rust unit tests use 64-byte synthetic binary fixtures; CI runs formatting, Clippy, tests, and a release build, with no AU0 parser or real-output AU0 check.

Therefore this repository may provide a hypothesis that a specific DEE build retains a native mode-21 branch, but it does not explain or repair an AU0 failure. The user-reported damaged output remains `USER_REPORTED_FAILURE_FILE_ID_UNBOUND); no output path/hash was supplied. `IMPLEMENTATION_READY=NO`.
