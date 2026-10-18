# CurrentFeatures math provider fixtures

These provider files are copied byte-for-byte from the approved `IMPLEMENTER_PACKAGE` after validating its complete package manifest identity. Tests reuse the single tracked package `MANIFEST.json` in `../f01e_provider/MANIFEST.json`; this directory intentionally contains no second manifest fixture.

| Fixture | Approved package path | Bytes | SHA-256 |
|---|---|---:|---|
| `feature-processor-math-v1.json` | `parameters/feature-processor-math-v1.json` | 35,441 | `c2cc80eed698285a7a022324bfed36e04c3b6e682cb21674b6316b3ec6a2ec0b` |
| `feature-fft4096-math-v1.json` | `parameters/feature-fft4096-math-v1.json` | 279,305 | `aea85e4fb5dcb0354ac11790466d324f1020a8bf5db3f9d981bc46755751fb33` |

This is internal-only test data. It contains no private key or authentication material, is not referenced by production code, and is not evidence of product audio acceptance.
