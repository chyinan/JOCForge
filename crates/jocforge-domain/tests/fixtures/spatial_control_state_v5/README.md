# Spatial control state v5 input-only vectors

Internal-only clean-room test input copied byte-for-byte from the approved supplement after its full manifest identity was verified:

| Fixture | Bytes | SHA-256 |
|---|---:|---|
| `INPUT-ONLY-VECTORS.json` | 2387 | `c5292f9a4dfd607704f4a442963b56b3f239e0bc69b8a9d03619fc65bdbc0d69` |

The fixture contains no key or authentication material. Tests compute actual results before comparison; expected values are never implementation branches. Do not read or copy files referenced by `research_basis`.

`PROFILE-IDENTITY-MISMATCH-VECTORS.json` is a separate local negative boundary derived from the approved profile-binding rule. It holds the F01E/F03 receipt lineages, state hash, and generation constant while changing only the profile identity. It does not add a supported profile and is not part of the approved supplement manifest.
