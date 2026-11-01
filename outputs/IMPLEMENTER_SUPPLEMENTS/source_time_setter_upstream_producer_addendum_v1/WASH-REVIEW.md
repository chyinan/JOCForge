# Wash review: source-time setter fields

Evidence-only supplement from a Ghidra static package. Source manifest SHA-256 is recorded in MANIFEST.json.

- A 120-byte source record is copied into a storage-master object. Input qword slots 10 and 11 (byte offsets 80 and 88) feed two master fields later read as binary64 seconds and normalized under the labels offset and FFOA.
- The values observed for R138 are both 3610.0 seconds, but equality does not identify the writer or join these fields to the R138 embedded-timecode conversion.
- No direct setter caller was found. The input-record writer, setter caller, same-owner/generation R138 invocation, and resolved timecode rate remain UNBOUND/NOT_CAPTURED. Gate 1 is not closed.
- A bounded observation plan is provided in the source README. It records only slot 10/11 scalars and joins setter/getter values through the same explicit owner and job/generation; it does not dump the source record or media payload.

This narrows the data representation and exact missing join; it does not authorize an inferred offset/FFOA rule for implementation.
