# Native measurement-to-serializer owner observation

## What was observed

One bounded run preserved the R138 measurement settings and omitted custom_dialnorm. DEE logged final integrated loudness -27.774199 and measured_loudness=-27.77; the unit was not stated. In the same run, one owner U16 value 28 was copied by the owner setter into staging and consumed by a 5-bit serializer call as 28.

This closes the same-owner field -> setter staging -> serializer-consumer edge for this run. It does not identify that field as an F06 result field or prove the measurement caused its value.

## Output metadata

The temporary output was 135,168 bytes with SHA-256 b85a77ac531aae0fe34954de9272a0f92d651f86076944118ca0442188df8e7b. Post-hoc parsing found 33/33 AUs with JOC signaling. Deployed-compatibility passed 33/33; ETSI strict validation failed 33/33. No spatial render or player playback was run. This is a DEE reference output, not a JOCForge result.

## Wash disposition

The observed U16 and 5-bit use are facts. A loudness-to-U16 formula remains a hypothesis: the matching numeric values and earlier non-controlled runs do not close typed source-member or generation lineage. Do not implement clamp(round(abs(loudness)),0,31) from these observations. The controlled -27.6 rounding discriminator was not run.

The source package was independently verified in normal and optimized Python modes. This Wash package contains no media, PCM, binary, raw log, license contents, absolute path, or module coordinates.
