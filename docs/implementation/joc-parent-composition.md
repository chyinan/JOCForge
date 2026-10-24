# JOC parent composition

Status: `LOCALLY_VERIFIED_COMPONENT_PROPOSALS__PROGRAMME_INSTALL_NOT_RUN`.

The implementation is bound to the washed `J05-PARENT-ASSEMBLY-REFERENCES-V1` package (manifest SHA-256 `c56d839b45d5e22440ba1a75ef7386b011c8f1c5cd76e9f71be1dcd0bcd68863`) and `JOC-FIT-STATE-SEQUENCE-REFERENCES-V1` (`b32626f382edbb4ea27051ece69431f02253f313d939a48ece94a58d42d65425`). Every listed payload matched its manifest before implementation.

`spatial_audio_composer` validates Type10 and reader-target receipts before receiver work. For g0/g1 it advances only the component/LFE portion of the complete F04 root. For g>=2 it binds component content g, LFE content g-2, reader envelope g/invocation g-2 and all 16 delivered producer generations, then executes receiver and mode0 fitting once. It returns a canonical full-root after proposal and an immutable J05 projection containing actual target/QMF/final-six buffers and receipt identities. It has no local commit API.

The J05 projection feeds temporal, geometry, 180-cell fit, representation retry and ID14 staging. Temporal family6 and geometry family7 use their approved byte encodings; cold SHA-256 values are `afa3ea6e59fb39634aa09884412c07dbeb6a3f081899837dc342a1f6f3205534` and `e575bf3985b08db4829cfe7ae78928a448b3eaab0e06996be7cdd9a814fc6ac3`. These component digests are not a whole JOC root digest.

Sequence state is independent of callback/source generation. Cold is -1; successful wire values are `0,0,1,...,1023,1,...`. All retries use the same value and only a complete body proposes the next sequence. Clip gain uses the explicit same-callback F04D selected index and target gain. Full/sparse/channel arrays remain callback-local scratch.

The selected JOC owner root now uses the separately approved `JOCSEL01` overlay (washed manifest SHA-256 `2bc9de24fc3c971477c9fe053d8e9dd9adb25e3c6e0dec8e00b2f13ffa6e3856`). It serializes next dispatch, I64 sequence, successful/writer counts, fit generation, retired flag, temporal and geometry in exactly 22,790 bytes; cold SHA-256 is `a90bc4b2e80fb1e226491e6e05ae35e1c2e50c661c622c19c9e9b40dcf0c1862`. Cold and five nonzero structural vectors pass. Historical `JOCOWN01` warm compatibility is not required for this selected internal root.

The remaining boundary is the 12-root programme parent. It must install this complete JOC after-state with every other source owner once and carry the actual ID14 body into F13 in the same transaction. F04C remains the only receiver/QMF executor and F13 remains the only FIFO executor; neither state is copied into JOCSEL01.
