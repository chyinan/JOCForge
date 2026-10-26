# Measurement record selection input recipes

These synthetic input-only recipes complete the aggregation inputs omitted by the earlier preferred supplement. Each case supplies the initial opaque 44-byte record, separate I32 member, F32 comparator bits, and every candidate's opaque record bytes, separate member, and F32 loudness bits. No native record-field offset is asserted.

Apply the already washed rule: initialize the record and member to zero and the finite F32 comparator to -120; for each candidate in order, replace the whole record, its separate member, and comparator only when candidate loudness is strictly greater. Equality retains the earlier record. Convert the final member to ID1 preferred I32 by exact equality to 1. No expected production body, real measurement, or selected-job value is supplied.

The cases cover no winner at the initial comparator, a strict winner followed by a tie, a later strict winner, and an empty candidate list. They exercise local selection and value association only; provider semantics, native record layout, selected backend, and runtime epoch remain separately unresolved.

Authority: F13-ID1-PREFERRED-INTERNAL-PROVIDER-BOUNDARY-V1, manifest SHA-256 9076bc0af478d15e6a3949e37b199d3435693fa6b2397e8868644bf0dfac372b. Native and product execution are NOT_RUN.
