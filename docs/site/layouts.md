# Layouts and objects

| CLI layout | Reconstruction carriers | Phase signalling | Default total bitrate |
|---|---|---|---|
| idx0 | 5.X | Off | 1024 kbps |
| idx1 (`flat7x`) | Flat 7.X with rear surrounds | Off | 1664 kbps |
| idx2 | 5.X+2 with top-front left and right | Off | 1664 kbps |
| idx3 (default) | 5.X | On | 1024 kbps |
| idx4 | 5.X+2 with top-front left and right | On | 1664 kbps |

Five-carrier audio accepts `--data-rate` values of 384, 448, 576, 640, 768 and 1024 kbps. Seven-carrier audio accepts `--total-rate` values of 1024, 1088, 1216, 1280, 1408 and 1664 kbps. The total includes the 640 kbps AC-3 core and the E-AC-3 dependent stream.

## Encode Flat 7.1

Pass absolute paths. Raw seven-carrier exports require a fresh empty directory and the suffix `.candidate.ec3`.

```powershell
$adm = (Resolve-Path "D:/Audio/input.adm.wav").Path
$out = (New-Item -ItemType Directory -Path (Join-Path $PWD ("flat-" + [guid]::NewGuid()))).FullName
.\target\release\jocforge.exe encode "$adm" (Join-Path $out "mix.candidate.ec3") --joc-layout idx1 --total-rate 1280
```

## Object motion and count

JOCForge follows Cartesian object positions at 1536-sample intervals (32 ms at 48 kHz), using jump and interpolation metadata for transitions. Bed channels keep their assigned positions.

Native EC-3 output supports experimental `--joc-objects 8` and `12` at all six rates for each profile. The default is 15. LFE is counted separately. Reduced modes group neighbouring slots before fitting coefficients and emitting their positions.

## Playback compatibility

Public 5.1 output plays silently on the tested iOS system decoder owing to HMAC verification. Flat 7.1 is experimental. The 1664 kbps Flat 7.1 test of *Love Song* yielded 8,417 access units and a complete OpenJOC 7.1 render. See [containers](containers.md) for tested playback paths.
