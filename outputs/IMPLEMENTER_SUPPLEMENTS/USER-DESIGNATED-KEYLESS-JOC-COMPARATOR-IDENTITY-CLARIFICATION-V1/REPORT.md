# Keyless Python comparator identity clarification

## Comparator and input identity

The user-designated comparator is `outputs/AUDIO_COMPATIBILITY_PROBE/candidate-keyless-compatible-joc.ec3`. Its current streaming SHA-256 is `383d7de4a57fd9bb32b8f871af1d2b5b0ad686a4b25e5fc6e51cd1be3adf6e86`; length is 17,395,712 bytes. The stream was not decoded, played, copied, or parsed.

The user confirms that the Python prototype's original ADM is the same authorized “人是猫” ADM, identity `authorized_adm_bwf_06`, SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. The sealed input crosswalk binds that authorized ADM identity. Do not retain “ADM identity unknown” as a blocker.

## Provenance limit

The adjacent compatibility receipt reports `private_hmac_key_used=false` and binds the candidate hash/length, but the probe folder has no sealed Python run receipt, prototype source hash, or recorded Python-run input SHA. Keep only that Python-run/output-to-input receipt relation as `USER_ASSERTED_NOT_RESEALED`. It does not block the requested implementation-side exact hash comparison.

The protected DEE-matching candidate was excluded and not opened. Do not compare against it.

## MVP comparison

After a JOCForge run finishes with application exit code 0, durable application receipt, and final file metadata, stream-hash the produced file and compare SHA-256 and byte length with the comparator above. Exact equality is the user's MVP criterion. Metadata-only JOC detection or an output from an exit-2 attempt does not satisfy it. Do not read comparator or output payload contents.
