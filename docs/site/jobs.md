# XML jobs and loudness

Run `jocforge encode --job-xml job.xml --preflight-only` to check the input, then omit `--preflight-only` to encode. The XML reader accepts ADM BWF under `adm` and `atmos_mezz`. Default output is 5.1 EC-3. For Flat 7.1 select `--joc-layout flat7x` and specify the D0 rate in `data_rate`. For CMAF select the container and total rate on the CLI, set `data_rate` to 640 and put the MP4 name under `output/ec3`.

## Timing

Inside `filter/audio/encode_to_atmos_ddp`, this example selects two seconds, prepends four seconds and appends one. Frame rounding gives 7.008 seconds of output.

```xml
<time_base>file_position</time_base>
<timecode_frame_rate>25</timecode_frame_rate>
<start>00:01:00:00</start>
<end>00:01:02:00</end>
<prepend_silence_duration>4</prepend_silence_duration>
<append_silence_duration>1</append_silence_duration>
```

Start and end may be set independently. Decimal timecodes use a half-open interval. Frame timecodes require a frame rate. An integer end-frame number includes that frame. Silence durations accept fractional seconds. Metering, encoding and object positions share the edited timeline.

## Audio metadata

Line and RF DRC independently accept `film_light`, `film_standard`, `music_standard`, `music_light` and `none`. Four mix levels and the preferred downmix mode are written to the stream. Metering supports ungated BS.1770-1, gated BS.1770-2/3/4 and LeqA. With dialogue detection enabled, the speech threshold selects dialogue or programme loudness. Surround and height trims are independent.

## Metering cache

The CLI reports `Loudness cache hit` when content, trim, silence and measurement settings match. Reuse spans bitrates and 5.1/Flat 7.1. Changes to the algorithm or its parameters invalidate a measurement. Override the directory with `JOCFORGE_CACHE_DIR`. Defaults are `%LOCALAPPDATA%/JOCForge/adm-loudness-v1` on Windows and `~/Library/Caches/JOCForge/adm-loudness-v1` on macOS.
