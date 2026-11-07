# ADM CHNA to AXML slot coverage

Two authorized inputs have exact one-to-one metadata slot coverage: one with N=36 and one with N=38. For each input, fmt channels, CHNA tracks/UIDs/capacity, and trackIndex range match N; every CHNA UID has one AXML definition; each slot resolves one direct channel-format ID and the full set is covered uniquely. CHNA packRef is recorded separately because a pack may contain multiple channel formats.

The inspection read RIFF/WAVE headers and fmt, chna, and axml chunks. It skipped data payload bytes and did not compute whole-file hashes. This proves only input metadata mapping for these two files. DEE runtime acceptance and U/K owner identity were not tested.
