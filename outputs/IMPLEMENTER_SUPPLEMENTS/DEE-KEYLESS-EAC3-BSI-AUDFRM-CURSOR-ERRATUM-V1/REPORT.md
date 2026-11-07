# E-AC-3 BSI-to-audfrm cursor correction

Wash independently recomputed the source and parent manifest hashes and reproduced the two first-frame BSI/audfrm cursor parses in normal Python and python -O. This supersedes the v2 claim that bit 113 was frmfgaincode.

For the hash-bound keyless candidate and historical Rust attempt-10 output, the first frame's BSI ends and audfrm starts at bit 113. At that bit `expstre` is 0 in the candidate and 1 in attempt-10. `frmfgaincode` is at bit 121 and is 1 in both. The two exponent-strategy branches advance to audfrm ends at bits 198 and 233 respectively; the observed difference follows the conditional field widths. [ETSI TS 102 366 V1.4.1 Annex E](https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf) defines the two expstre syntax choices.

The parser stops at audfrm end. It does not validate the full audblk or decoder behavior. The user's current Audition-failure file is not linked by path/run/hash to historical attempt-10, whose reported executable differs from the file now present at that path. Candidate GENERATION.json is an unsealed record; candidate-to-ADM identity and recovered-ADM completeness remain UNBOUND/UNVERIFIED. Do not attribute Audition failure or an audio algorithm defect to this bit difference.

No raw encoded bytes, PCM, key/license, or native source/decompilation are included.
