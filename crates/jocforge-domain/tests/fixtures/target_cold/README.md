# F03 target-cold provider fixture

Internal-only clean-room test input copied byte-for-byte from the approved `IMPLEMENTER_PACKAGE`:

| Fixture | Approved source | Bytes | SHA-256 |
|---|---|---:|---|
| `f03-selected-target-cold-v1.json` | `parameters/f03-selected-target-cold-v1.json` | 3139 | `a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00` |

Tests reuse the single tracked package `MANIFEST.json` at `../f01e_provider/MANIFEST.json`; no second manifest fixture is maintained. This file is test data only and contains no private key or authentication material. Its use and distribution remain limited to the package's internal implementation scope.
