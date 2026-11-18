# User Command Timeline

## 2026-11-15 18:21:54 +08:00 — Official demo OpenJOC AU0 metadata inspection

- **Status:** Preflight complete; one demux/inspect attempt pending.
- **Purpose:** Inspect only sanitized AU0 metadata from the user-designated Dolby Atmos DD+ 7.1.4 official demo: EMDF ID11/ID14 configuration scalars, payload order, D0 block-2 skipfld ordinal/length if exposed, chanmap, JOC profile/object count/complexity. No decode or render.
- **Input:** `C:\Users\chyinan\Downloads\Dolby-Atmos-DD+ 7.1.4.mka`.
- **Track identification:** `mkvmerge -J` exit 0; one audio track total and exactly one E-AC-3 track: ID 0, `A_EAC3`, 8 channels, 48 kHz.
- **Output directory:** `D:\JOCForge-attempts\official-demo-openjoc-20261115-01\` (verified absent before this entry).
- **Temporary track:** `D:\JOCForge-attempts\official-demo-openjoc-20261115-01\temp\track-0.ec3`.
- **Retained result paths:** `D:\JOCForge-attempts\official-demo-openjoc-20261115-01\AU0-METADATA.json` and `D:\JOCForge-attempts\official-demo-openjoc-20261115-01\COMMAND-LOG.txt`.
- **Tools:** MKVToolNix `mkvmerge/mkvextract v103.0`; OpenJOC `0.20.1`, executable SHA-256 `2e84afcfa5e197332a84f9386ffa4c69f0c12e8ca5cf95c23690af320cfa1a39`. `inspect --help` lists raw EC3 and seekable ordinary MP4/M4A input, so MKA will be demuxed to the unique temporary EC3 path.
- **Tool identities:** `mkvmerge.exe` SHA-256 `e66056a12f5b99a43c37f11374a0b08648a1cef7a0c99e0c15786e95abfc5ddd`; `mkvextract.exe` SHA-256 `a7e8b7358c743cbd4d5261ff0d09b1ae0538a5825266a31110c00d978ae36403`.
- **Idle process snapshot:** 2026-11-15 18:21:54 +08:00; no `dee`, `openjoc`, `mkvmerge`, or `mkvextract` processes found.
- **Authorization:** Continuation of the user-authorized official-demo structural inspection; the current task authorizes this one demux/inspect attempt without a new Fresh GO.
- **Storage scope:** The temporary EC3 exists only for this attempt and will be deleted after a successful inspect and sanitized AU0 extraction. The retained JSON will contain only approved scalar fields; no EC3, MKA, WAV, PCM, decoded samples, or payload bytes are retained.

### Pre-launch check

- Timestamp: 2026-11-15 18:23:56 +08:00
- Process snapshot: [] (dee/openjoc/mkvmerge/mkvextract)
- Unique output and temp paths: absent immediately before creation.
- Planned single track: mkvextract track ID 0 to temp/track-0.ec3, then OpenJOC inspect with --au-range 0:0.


### Demux/inspect attempt start

- Timestamp: 2026-11-15 18:26:58 +08:00
- mkvmerge -J result: exit 0; one E-AC-3 audio track (ID 0 / A_EAC3 / 8 ch / 48 kHz).
- Idle process snapshot at launch boundary: none among dee, openjoc, mkvmerge, mkvextract.
- One attempt only; no fallback to another track or rerun.


### OpenJOC launch boundary

- Timestamp: 2026-11-15 18:30:24 +08:00
- Process snapshot: [] (dee/openjoc/mkvmerge/mkvextract).
- Demux result: success for the unique E-AC-3 track 0; temporary track exists at the registered path.
- Planned OpenJOC invocation: inspect <temp EC3> --json --aus --objects --emdf --au-range 0:0; output will be sanitized in memory to AU0 fields.


### Completion

- OpenJOC inspect: exit 0; retained exactly one AU0 detail.
- Sanitized metadata: D:\JOCForge-attempts\official-demo-openjoc-20261115-01\AU0-METADATA.json (SHA-256 74276a01fd01cbe8df1ea12ed8a65a3185f356f913812c2464b205bce425cace).
- Raw inspect JSON persisted: no. Temporary EC3 deleted: true.
- AU0 payload IDs: 11,14; EMDF ID11/14 first-seen config records: 0.
- Chanmap/skipfld status: chanmap NOT_EXPOSED_IN_AU0_DETAIL; block-2 ordinal/C not exposed by aggregate inspect.


### Sanitization correction

- OpenJOC inspect completed with exit 0 and one AU0 detail; the retained metadata record is marked PARTIAL because the ID11/ID14 config-field extraction hit a PowerShell automatic process-ID variable collision.
- Raw inspect JSON was held in memory and not persisted; the single temporary EC3 was deleted after the inspect. No retry was run.
- Final sanitized metadata SHA-256: c0e0936017407a5b8aee97bd34b1033e464bf906dfa6489f98baf900608e12a2.


## 2026-11-15 E-1102 vs official AU0 OpenJOC comparison

- Preflight: unique existing E-1102 candidate path confirmed; candidate file bytes/hash were not read by shell.
- Official reference: sanitized AU0-METADATA.json SHA-256 c0e0936017407a5b8aee97bd34b1033e464bf906dfa6489f98baf900608e12a2.
- Tool: specified OpenJOC 0.20.1 SHA-256 2e84afcfa5e197332a84f9386ffa4c69f0c12e8ca5cf95c23690af320cfa1a39.
- Command: inspect <existing candidate EC3> --json --aus --objects --emdf --au-range 0:0.
- Compare output: D:\JOCForge-attempts\E1102-love-song-flat7x-20261115-c7dbd5e\openjoc-au0-compare-20261115-01.json.
- Process snapshot: [] (dee/openjoc/mkvmerge/mkvextract); timestamp 2026-11-15 18:40:14 +08:00.
- No DEE run or EC3 re-extraction; compare only the same AU0 access_units.payload_ids field.


## 2026-11-15 Official MKA OpenJOC retry attempt -02

- Status: Preflight registered; one attempt pending.
- Purpose: retain sanitized AU0 EMDF ID11/14 configuration, payload order, JOC profile/object count/CI, channel map and block-2 skipfld fields if exposed.
- Input: C:\Users\chyinan\Downloads\Dolby-Atmos-DD+ 7.1.4.mka.
- Planned output: D:\JOCForge-attempts\official-demo-openjoc-20261115-02; temporary EC3: D:\JOCForge-attempts\official-demo-openjoc-20261115-02\temp\track-0.ec3.
- Retained files after success: AU0-METADATA.json and COMMAND-LOG.txt only.
- Tools: mkvmerge v103.0 ('The Darkness') 64-bit (SHA-256 e66056a12f5b99a43c37f11374a0b08648a1cef7a0c99e0c15786e95abfc5ddd); mkvextract v103.0 ('The Darkness') 64-bit (SHA-256 a7e8b7358c743cbd4d5261ff0d09b1ae0538a5825266a31110c00d978ae36403); OpenJOC 0.20.1 (SHA-256 2e84afcfa5e197332a84f9386ffa4c69f0c12e8ca5cf95c23690af320cfa1a39).
- Process snapshot at 2026-11-15 18:43:09 +08:00: [] for dee/openjoc/mkvmerge/mkvextract.
- Authorization basis: continued user authorization for one official-sample metadata inspect; no new Fresh GO requested.
- No MKA content hash/read, no DEE, decode, render, or source edits.


- Track identification: mkvmerge -J exit 0; audio_track_count=1; unique A_EAC3 track_id=0, channels=8, sample_rate_hz=48000 at 2026-11-15 18:43:28 +08:00.

### Attempt -02 demux launch boundary

- Timestamp: 2026-11-15 18:43:50 +08:00.
- Process snapshot: [] for dee/openjoc/mkvmerge/mkvextract.
- Unique output/temp path confirmed absent; unique A_EAC3 track 0 was identified by mkvmerge -J.
- Next operation: one mkvextract tracks invocation; no retry.


- Attempt -02 pre-inspect snapshot at 2026-11-15 18:44:47 +08:00: [] for dee/openjoc/mkvmerge/mkvextract; track 0 demux succeeded; now launching OpenJOC once.

- Attempt -02 completion at 2026-11-15 18:46:34 +08:00: OpenJOC inspect exit 0, one AU0 detail, ID11/14 config status CAPTURED_FIRST_SEEN_AU0_VARIANTS, metadata SHA-256 f34c3c3db0a01b5c2cac5736ffe155943fb7143165f97931f6863a8481156064, temp EC3 deleted true; raw JSON not retained.


## 2026-11-15 Official MKA OpenJOC attempt -03

- Status: Preflight registered; track identification pending.
- Purpose: one read-only AU0 inspect preserving exact JSON fields access_units[0].payload_ids and emdf.payload_orders[0], plus ID11/14 config observations.
- Input: C:\Users\chyinan\Downloads\Dolby-Atmos-DD+ 7.1.4.mka; output: D:\JOCForge-attempts\official-demo-openjoc-20261115-03; temp: D:\JOCForge-attempts\official-demo-openjoc-20261115-03\temp\track-0.ec3.
- Tools: mkvmerge v103.0 ('The Darkness') 64-bit SHA-256 e66056a12f5b99a43c37f11374a0b08648a1cef7a0c99e0c15786e95abfc5ddd; mkvextract v103.0 ('The Darkness') 64-bit SHA-256 a7e8b7358c743cbd4d5261ff0d09b1ae0538a5825266a31110c00d978ae36403; OpenJOC 0.20.1 SHA-256 2e84afcfa5e197332a84f9386ffa4c69f0c12e8ca5cf95c23690af320cfa1a39.
- No-launch process snapshot at 2026-11-15 19:22:48 +08:00: [].
- No MKA hash or direct bytes read before demux; no DEE/decoder/render; no raw inspect JSON or EC3 retained.


- Attempt -03 track identification: mkvmerge -J exit 0, audio tracks=1, unique A_EAC3 ID 0, 8ch/48000Hz at 2026-11-15 19:23:04 +08:00.

### Attempt -03 demux boundary

- Timestamp: 2026-11-15 19:23:23 +08:00; process snapshot [] for dee/openjoc/mkvmerge/mkvextract.
- Exactly one A_EAC3 track identified: ID 0, 8 channels, 48000 Hz.
- One mkvextract call planned; no fallback/retry.


- Attempt -03 OpenJOC launch snapshot at 2026-11-15 19:24:20 +08:00: [] for dee/openjoc/mkvmerge/mkvextract; unique track 0 demux succeeded; exact fields access_units[0].payload_ids and emdf.payload_orders[0] will be retained separately.

- Attempt -03 completed at 2026-11-15 19:25:44 +08:00: inspect exit 0; exact access_units[0].payload_ids / emdf.payload_orders[0] fields stored separately; ID11/14 config observations stored; sanitized metadata SHA-256 aabd2d359d3a1ace38e1180e8e1b26e2501a51ab08eb71df628fed8ef2a04b51; temp EC3 deleted true; raw inspect JSON not retained.


## 2026-11-15 Official MKA OpenJOC full-stream attempt -04

- Status: Timeline pre-registered; track identification pending.
- Purpose: full-stream inspect, retaining only sanitized aggregate AU/I0/D0/chanmap/EMDF/config/JOC/validation fields.
- Input: C:\Users\chyinan\Downloads\Dolby-Atmos-DD+ 7.1.4.mka; output: D:\JOCForge-attempts\official-demo-openjoc-20261115-04; temporary track: D:\JOCForge-attempts\official-demo-openjoc-20261115-04\temp\track-0.ec3.
- Retained result paths: FULL-STREAM-METADATA.json and COMMAND-LOG.txt only.
- Tools: mkvmerge v103.0 ('The Darkness') 64-bit SHA-256 e66056a12f5b99a43c37f11374a0b08648a1cef7a0c99e0c15786e95abfc5ddd; mkvextract v103.0 ('The Darkness') 64-bit SHA-256 a7e8b7358c743cbd4d5261ff0d09b1ae0538a5825266a31110c00d978ae36403; OpenJOC 0.20.1 SHA-256 2e84afcfa5e197332a84f9386ffa4c69f0c12e8ca5cf95c23690af320cfa1a39.
- No-launch process snapshot at 2026-11-15 19:52:03 +08:00: [].
- No MKA content hash/read before demux; no DEE/decode/render; raw JSON/EC3 will not be retained.


- Attempt -04 track identification: mkvmerge -J exit 0; audio_count=1; unique A_EAC3 ID 0, 8ch/48000Hz at 2026-11-15 19:52:18 +08:00.

### Attempt -04 launch boundary

- Timestamp: 2026-11-15 19:53:23 +08:00; process snapshot [] for dee/openjoc/mkvmerge/mkvextract.
- Unique A_EAC3 track 0, 8 ch/48 kHz confirmed by mkvmerge -J.
- Full-stream OpenJOC inspect will retain only sanitized aggregate metadata; raw stdout remains in memory only.


- Attempt -04 inspect launch snapshot at 2026-11-15 19:54:19 +08:00: [] for dee/openjoc/mkvmerge/mkvextract; unique track 0 demux succeeded; full-stream inspect begins once.
- Attempt -04 full-stream inspect launch snapshot at 2026-11-15 19:58:52 +08:00: [] for dee/openjoc/mkvmerge/mkvextract; full inspect begins once.
- Attempt -04 full-stream inspect completed at 2026-11-15 19:59:17 +08:00: OpenJOC exit 0, aggregate-only metadata SHA-256 7b85a69aa112429eb8b7f07b6848ae2527a9e33091e5ea2e94914083ebd39afb, raw JSON not retained, temporary EC3 deleted true.
