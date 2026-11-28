# XML jobs and loudness

`jocforge encode --job-xml job.xml` reads the job instead of taking input and output paths on the command line. Add `--preflight-only` to check the XML and the input without encoding. The same flag also prints the sample count of each channel before and after the edit.

`adm` and `atmos_mezz` both read an ADM BWF. The parsed input keeps the type declared in the XML. The default encode is 5.1 EC-3.

In the embedded ADM XML, surrounding spaces, tabs or line breaks on object coordinates, spread, diffuse and jump interpolation length do not change their numeric values.

For Flat 7.1, add `--joc-layout flat7x`. In that job, `data_rate` is the D0 rate: `640` means a 1280 kbps total. The output path still has to be a `.candidate.ec3` in a new empty directory. For CMAF, select the container and the total rate on the CLI, set `data_rate` to 640, and put the `.mp4` name under `output/ec3`. See [Output containers](containers.md).

## Timing

Inside `filter/audio/encode_to_atmos_ddp`, the encoder can trim the ADM and add silence. This fragment keeps 00:01:00 through 00:01:02, prepends 4 seconds and appends 1 second. The programme is 7 seconds. EC-3 frames are 32 ms, so the file is 7.008 seconds.

```xml
<time_base>file_position</time_base>
<timecode_frame_rate>25</timecode_frame_rate>
<start>00:01:00:00</start>
<end>00:01:02:00</end>
<prepend_silence_duration>4</prepend_silence_duration>
<append_silence_duration>1</append_silence_duration>
```

- `start` and `end` are optional. The defaults are the beginning and the end of the file. The interval is half-open. A decimal form such as `00:01:00.250` is also accepted.
- `HH:MM:SS:FF` needs a frame rate. The encode element's `timecode_frame_rate` wins, then the input frame rate. Decimal timecodes may leave the rate as `not_indicated`. An integer end-frame number includes that frame.
- Silence durations are seconds and may be fractional. An omitted or empty value is zero. Metering, encoding and object positions share this edited timeline.

## Audio metadata

Line and RF DRC are independent. Each accepts `film_light`, `film_standard`, `music_standard`, `music_light` or `none`. The default pair keeps the encoder's existing output. Four downmix levels and the preferred downmix mode are written into the stream. For Flat 7.1 and legacy AC-3 carriage, the independently playable AC-3 core carries these settings as well.

Loudness metering can use ungated BS.1770-1, gated BS.1770-2, BS.1770-3, BS.1770-4 or LeqA. The current 5.1 meter shares one gating algorithm for BS.1770-2, -3 and -4. With dialogue detection on, Earshot marks speech. At or above the speech-ratio threshold the meter reports dialogue loudness. Below it, the meter reports programme loudness. LeqA includes the final partial meter block and measures only the programme samples, including explicitly requested silence but not internal callback padding.

Surround trim and height trim are independent. Each can use a preset from the XML. A manual value attenuates by position. When a preset and a manual value are both set, the combined attenuation is limited by the larger of the two ceilings.

## Loudness cache

The first encode measures the ADM. A later encode of the same content reuses that measurement across bitrates and across 5.1 and Flat 7.1, when the trim, the lead and tail silence, and the measurement settings match. A hit prints `Loudness cache hit`. A new measurement algorithm, or a change to its built-in parameters, measures again.

The cache stores the exact loudness, peak and gating values, plus a checksum. On Windows the default directory is `%LOCALAPPDATA%\JOCForge\adm-loudness-v1`. On macOS it is `~/Library/Caches/JOCForge/adm-loudness-v1`. Set `JOCFORGE_CACHE_DIR` to store measurements under `<your-directory>/adm-loudness-v1`. A relative directory is resolved from the process working directory, not from the ADM or XML file. Delete the directory to force a new measurement. If the directory is not writable, the current export still finishes.
