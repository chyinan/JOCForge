# Native embedded-timecode to sample-position evidence

## Scope

This is one bounded native DEE observation for an authorized ADM BWF. It records the selected `embedded_timecode` range and the native composite-master sample interval reported for that run. It is not a JOCForge parser implementation or a general frame-rate rule.

## Observed runs

Run-113 used XML start/end values 5,770 and 5,772. DEE rejected it because the computed start time 240.416667 seconds preceded the file's embedded start at 3,610 seconds. This failure provides no sample-position evidence.

Run-115 used `embedded_timecode` start/end values 86,640 and 86,664. The runtime input matched the authorized source hash. DEE completed encoding and reported native composite-master samples 0 through 49,999 inclusive, 50,000 samples total. The EC3 output was 135,168 bytes with SHA-256 `7ea9c29780a43b9bd14137f76ee06fb44f9c500fa8bccafda28706a85a2efe83`.

The relationship to 24 fps and 2,000 samples per frame is an inference consistent with the source DEE timing metadata and observed interval. XML did not explicitly specify `timecode_rate`. Do not promote that inference to an explicit task setting or universal rate rule.

The observer read no PCM and persisted no PCM. It captured no spatial callback or generation join. Model execution and uninstrumented output comparison are `NOT_RUN`.
