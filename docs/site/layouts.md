# Layouts and objects

`--joc-layout` selects the reconstruction carriers and whether phase signalling is on. Omit it to use idx3.

| Layout | Carriers | Phase | Default bitrate |
|---|---|---|---|
| `idx0` | 5.X | Off | 1024 kbps (`--data-rate`) |
| `idx1`, `flat7x` | Flat 7.X, rear surrounds | Off | 1664 kbps (`--total-rate`) |
| `idx2` | 5.X+2, top front left and right | Off | 1664 kbps (`--total-rate`) |
| `idx3` (default) | 5.X | On | 1024 kbps (`--data-rate`) |
| `idx4` | 5.X+2, top front left and right | On | 1664 kbps (`--total-rate`) |

Five-carrier layouts accept `--data-rate` values of 384, 448, 576, 640, 768 and 1024 kbps. Seven-carrier layouts accept `--total-rate` values of 1024, 1088, 1216, 1280, 1408 and 1664 kbps. A seven-carrier total includes a 640 kbps AC-3 core. The rest goes to the E-AC-3 dependent substream.

idx2 and idx4 use the same seven inputs: L, R, C, Ls, Rs, Tfl and Tfr. Height-bed channels go to the height pair. Object energy is split between the floor and the height pair from the object coordinates, and those seven channels are then fitted to JOC coefficients. The dependent channel map is `0x1810` (Ls, Rs and the height pair). The compatible core supplies LFE. The two layouts differ by the phase-signalling index.

idx0, idx2 and idx4 default to strict ETSI signalling. `--etsi-strict` also forces that signalling on native 5.1 and Flat 7.1. Carriage and the EMDF profile are set on [Substreams and signalling](substreams.md).

## Objects

JOCForge reads Cartesian positions every 1536 samples, which is 32 ms at 48 kHz. An object may contain several `audioBlockFormat` blocks. Transitions follow `jumpPosition` and `interpolationLength`. Bed channels keep their speaker positions.

On native EC-3, `--joc-objects` accepts `8` and `12` at every rate of every profile. The default is 15. LFE is not included in that count. The reduced modes merge neighbouring object slots before fitting coefficients, and they write the merged positions. Older OpenJOC builds use a different stereo render path for the reduced counts than for the default 15.

## Examples

Pass absolute paths. `$PWD` works in PowerShell and POSIX shells. A raw seven-carrier file must use the `.candidate.ec3` suffix and a new empty directory. idx0 can name the output file directly.

Flat 7.1 at 1280 kbps:

```sh
mkdir flat7x
jocforge encode "$PWD/input.adm.wav" "$PWD/flat7x/mix.candidate.ec3" --joc-layout flat7x --total-rate 1280
```

5.X with phase off:

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/music-idx0.ec3" --joc-layout idx0
```

Height pair, with phase off and on:

```sh
mkdir height-pair
mkdir height-phase
jocforge encode "$PWD/input.adm.wav" "$PWD/height-pair/song.candidate.ec3" --joc-layout idx2
jocforge encode "$PWD/input.adm.wav" "$PWD/height-phase/music.candidate.ec3" --joc-layout idx4
```

## Playback

The tested iOS system decoder stays silent on JOCForge 5.1 output. That decode needs an HMAC key, and this repository does not ship the key.

Flat 7.1 is experimental. A 4:29 encode of *Love Song* at 1664 kbps produced 8,417 access units. OpenJOC 0.20.2 accepted every unit in compatibility mode and rendered 7.1. The same six Flat 7.1 rates were also encoded from one 48 kHz DaVinci ADM master with moving objects, decoded with FFmpeg and rendered to 7.1 in OpenJOC. The deployed-compatibility check passed. The strict ETSI check still reports a `codecdatae` difference on payloads 11 and 14.

Player checks for the remuxed files are on [Output containers](containers.md).
