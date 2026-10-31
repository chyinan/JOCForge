# Reader origin, cursor, codec, and writer addendum

This Wash-only addendum records one bounded R138 native call. The source ADM BWF and the English-path alias have the same SHA-256; the alias was used because the native tool rejects the original non-ASCII path. The original source was not changed.

For the selected call, the configured absolute start boundary was 173,280,000 samples at 48 kHz, the relative start sample was 0, and the reader cursor before the request was 6,144. Their sum gives a **derived** absolute ADM sample position of 173,286,144 for that call. No native absolute-position field was captured. The normalized offset's upstream owner and the general offset/FFOA rule remain open.

The same native caller activation binds the reader's descriptor to the codec input, then binds a 4,096-byte codec output buffer to the writer input and its normal return. Static analysis identifies the writer ordinal as an output-stream routing value; it is not a generation identifier. Reader cursor, ADM sample position, host frame, spatial callback, fit generation, U04 invocation, and consumer generation remain distinct domains.

The package contains only normalized scalar values, opaque identity tokens, hashes, and verification code. It contains no audio/PCM, raw pointers, addresses, binary excerpts, installation paths, or license contents. Model execution, model comparison, Type10/six-plane composition, and Gate 1 closure were not performed by this addendum.

Verify with:

```text
python verify_reader_origin_cursor_spatial_commit.py
python -O verify_reader_origin_cursor_spatial_commit.py
```
