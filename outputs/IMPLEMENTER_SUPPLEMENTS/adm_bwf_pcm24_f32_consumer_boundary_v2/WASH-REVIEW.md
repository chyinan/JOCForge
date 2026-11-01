# Wash review: PCM24 reader to first F32 consumer (V2)

The bounded Ghidra chain reaches a conditional three-byte BWF source-read branch. When bytes-per-sample equals 3, it reads three bytes per sample and writes a short through a high-word/bitwise-carry operation. This establishes a conditional PCM24-to-S16 narrowing branch only; it is not a verified PCM24-to-F32 normalization rule.

A separate helper returns float from a four-byte input path, with no call edge from the three-byte branch. The sample-6 job has not been dynamically joined to either branch, and no typed path reaches the first DDP F32 consumer. Scale, precision and rounding remain unbound.

The approved JOCForge input contract separately specifies U24LE signed PCM24 byte addressing. It does not make this static conditional short branch a DEE algorithm. The minimum follow-up, if required, is one same-generation nonzero sample and its first-DSP F32 bit pattern with typed channel/row identity; discard the value immediately. A sample that does not hit a tie boundary cannot establish tie rounding.
