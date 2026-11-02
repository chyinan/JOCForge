# Hanmao same-run aggregate scalar to BSI U5

## Scoped result

For the authorized `人是猫` ADM input SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`, existing receipts bind one successful DEE attempt (`a-fresh-reference`, PID 35004, exit code 0) to aggregate scalar `-20.665958` and a bounded header U5 observation of `21` in the same attempt. The U5 field was reported at bit offset 45, width 5 (`-21 dB`); the helper read seven prefix bytes and zero payload bytes, and retained no raw prefix.

The candidate F12 rule, under the explicit assumption that the scalar is `selected_lufs`, predicts U5 `21`, matching this same-run observation. This closes the **receipt-level input → scalar → U5 association** for this sample and supports a focused implementation trial. The native scalar unit is not stated, and the measurement-to-BSI owner/generation edge is still not closed. Keep those limits visible; they do not block an MVP trial of the observed rule.

## Confidence and source integrity

The linked multi-ADM support package gives the scoped candidate formula confidence `0.82`: three different authorized ADM sources have observed U5 values matching the formula; a fourth has a scalar-only prediction. The Hanmao same-run receipt is the direct same-attempt instance for this exact input.

One provenance discrepancy was found during Wash: the incoming evidence says its prior cross-input manifest SHA-256 was `833823ee48a1380be746673da8c555397dd97e7b8fe2068257f7fe43dfa06134`, while the currently available `outputs/multi-adm-dialnorm-cross-input/MANIFEST.json` hashes to `439df32fce6b2dfe8f20026225536ef1e954784d1e99fd50d94e1a0b27b2ed33`. The referenced prior evidence JSON and input-vector payload hashes do match the currently available files (`a9283065...` and `cd98443b...`). The supplement preserves this manifest-reference mismatch rather than normalizing it away.

No new DEE run or input rehash was performed. No payload body, raw prefix, PCM, private key, or license material is included.

## Recommended implementation trial

Use the existing scoped candidate for the F12S U5 field on `人是猫`: treat the aggregate scalar as candidate `selected_lufs`, apply the already hash-bound F12 formula, and verify that the produced U5 is `21` at the five-bit field. Keep the field mapping and observed same-run receipt as separate facts; do not claim a universal native unit from this one scalar.
