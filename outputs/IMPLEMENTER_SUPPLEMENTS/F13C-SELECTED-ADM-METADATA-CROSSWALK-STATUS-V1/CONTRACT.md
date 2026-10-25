# Selected ADM metadata and native profile inputs

Status: `ADM_METADATA_CAPTURE_BOUNDED; NATIVE_PROFILE_INPUT_CROSSWALK_UNBOUND`.

The approved package identifies `test-vectors/full-song/input-adm.wav` as 48 kHz, 38-channel, 24-bit PCM. A Wash-only metadata receipt reports that its AXML contains DirectSpeakers and Objects semantic type labels, and its CHNA binds 38 unique track indices. These are ADM semantic/track facts; they do not identify the native profile type, profile-row identity, `a7`, partial-block flag, or native list selector.

The receipt reports reading only the RIFF headers, `fmt`, AXML, and CHNA ranges. The 743,611,284-byte PCM `data` payload was seek-skipped and read as 0 bytes. The DBMD payload was not inspected. The actual full-file digest was not recomputed; the package only binds to the digest declared by the approved package manifest. Of the 84,597 bytes reported read, hashes were retained for the 84,521-byte AXML and CHNA payloads; hashes for 76 bytes of container/chunk headers were not retained at the original read and were not reconstructed.

The AXML/CHNA receipt's chunk digests are reported from the original observation and were not replayed by Wash. The receipt itself and its normal/optimized checker were verified; all eight listed approved JSON vector identities were independently checked against the approved package manifest.

## Open mapping

The inspected metadata includes semantic labels, track bindings, and time fields, but supplies no approved mapping from those fields to the native I32 profile pair list/key-1 type, profile-row owner, `a7` value/unit/origin, partial-block flag, or selected list selector. Do not infer the native fields from semantic type labels, channel/object counts, timestamps, sample count, or jump interpolation. This result is limited to the inspected RIFF/fmt, AXML, and CHNA fields; it makes no claim about the uninspected DBMD payload or the whole native pipeline.

No algorithm or selected-profile producer rule is established. Keep the input-to-native-profile mapping `UNBOUND` until the host/constructor path joins these inputs to the native descriptor and row identity.
