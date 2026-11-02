# Sample 6 full-output validation with scope-correct references

The current authorized JOCForge run produced a 17,395,712-byte output with
SHA-256 `d9680298ac1d913f408b77a72b45171fc379dabb71de80e44c24d27cbf1de6da`.
FFprobe parses the full file as 48 kHz, 1024 kb/s, 5.1(side) E-AC-3 JOC;
FFmpeg decode-only exits 0 with no errors. PCM was streamed in memory and not
persisted.

Scope correction for DialNorm: Rust's full-file first-AU U5 is 17. The DEE U5
value 21 comes from a bounded file-position window 2160–2208 with an unstated
scalar unit, not from a full-file DEE reference. Therefore these values must
not be called a same-scope full-file mismatch. A full-file DEE U5 and decoded
audio comparison remain NOT_RUN.

The BSI views need investigation: FFprobe exposes downmix fields as zero,
while an independent bounded 10-byte header read computes the configured bit
slice as mixmdate=1, dmixmod=2 and four level codes=4. The raw bits are
hash-bound to this output; the semantic discrepancy with FFprobe is OPEN, not
yet evidence of a writer bug or DEE mismatch.

The keyless DEE attempt remains NOT_RUN because DEE auto-discovers product
licenses and no safe license-free invocation was found. No license/key was
used, no historical Python/control group was read, and no PCM or encoded file
is included in this Wash package.
