# Output containers

## Fragmented MP4 (CMAF)

`--container cmaf` writes one initialization segment followed by a fragment for each 1536-sample access unit, all in a single MP4 file. All five profiles use strict EMDF, six blocks and I0+D0. Choose a total bitrate of 1024, 1280, 1408, 1536 or 1664 kbps (default). D0 receives 640 kbps. LFE defaults to `none`. Select `--lfe-owner independent`, `dependent` or `replace-independent` for I0, supplemental D0 or replacement D0.

```powershell
jocforge encode "D:/Audio/input.adm.wav" "D:/Audio/mix.mp4" --joc-layout idx4 --container cmaf --total-rate 1664
```

## MKA and conventional MP4

Use FFmpeg stream copy on the encoded EC-3 file. Select the MP4 muxer explicitly for an M4A filename.

```sh
ffmpeg -n -i mix.ec3 -map 0:a:0 -c:a copy -f matroska mix.mka
ffmpeg -n -i mix.ec3 -map 0:a:0 -c:a copy -f mp4 mix.m4a
```

The tested Flat 7.1 MKA opened in Adobe Audition and Windows Media Player. The tested Flat 7.1 M4A opened with the iOS system decoder.

![Flat 7.1 in Adobe Audition](assets/screenshot.png)
