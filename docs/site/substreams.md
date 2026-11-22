# Substreams and signalling

I0 carries the independent presentation. D0 and later dependents can supplement or replace channels in that presentation.

| Option | Values | Effect |
|---|---|---|
| `--dependent-count` | 0–5 for idx0 and idx3; 1–6 for idx1, idx2 and idx4 | Set the number of dependent substreams. Rear pairs and height pairs stay together. |
| `--lfe-owner` | `independent`, `dependent`, `replace-independent`, `none` | Take LFE from I0, add it on D0, replace the I0 LFE with D0, or omit LFE. D0 ownership needs a dependent. |
| `--block-partition` | `6` (default), `3+3`, `2+2+2`, `1x6` | Split one 1536-sample interval into equal groups. `1x6` may also be written `1+1+1+1+1+1`. |
| `--carriage` | `skipfld`, `auxdata` (alias `etsi`), `legacy-ac3` | Put EMDF in an audio block, put it at the frame tail, or use a legacy AC-3 core. |
| `--emdf-profile` | `etsi`, `deployed` | Choose strict ETSI or deployed-compatible signalling. This is independent of carriage. |

Setting a dependent count or an LFE owner selects native E-AC-3: I0 plus D0…Dn, six audio blocks per frame. The last dependent carries JOC/OAMD. With zero dependents, I0 carries it. The programme rate is split across those substreams. Leaving both options unset keeps the original encode path.

If only LFE ownership is set, five-carrier layouts default to zero dependents and seven-carrier layouts default to one. A raw seven-carrier file still uses `.candidate.ec3` in its own empty directory. File input and XML jobs take the same options.

| `--lfe-owner` | I0 LFE | D0 LFE | LFE after decode |
|---|---|---|---|
| `independent` (default) | Present | Absent | I0 |
| `dependent` | Absent | Present | D0 supplements I0 |
| `replace-independent` | Present | Present | D0 replaces I0 |
| `none` | Absent | Absent | None |

Five dependents on 5.X. D0 LFE replaces I0, and D4 carries JOC:

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/mix.ec3" --joc-layout idx0 --dependent-count 5 --lfe-owner replace-independent
```

Six dependents on 5.X+2. D0 supplements LFE, and D5 carries the height pair and JOC:

```sh
mkdir height-multi
jocforge encode "$PWD/input.adm.wav" "$PWD/height-multi/mix.candidate.ec3" --joc-layout idx4 --total-rate 1664 --dependent-count 6 --lfe-owner dependent
```

## Block partition

The default partition is `6`: one group covering the whole 1536-sample interval. `3+3`, `2+2+2` and `1x6` split that interval into equal groups. Duration and total rate stay on the original 1536-sample clock. Each group writes I0, then its own D0…Dn. The last substream of the last group carries JOC/OAMD once. If the rate cannot hold the syncframe headers, exponents and metadata, encode returns a capacity error.

```sh
mkdir height-short
jocforge encode "$PWD/input.adm.wav" "$PWD/height-short/song.candidate.ec3" --joc-layout idx4 --total-rate 1664 --block-partition 3+3 --dependent-count 2 --lfe-owner dependent
```

## EMDF carriage and profile

`--carriage skipfld` places the protected EMDF container in an audio block. `--carriage auxdata` places it at the tail of the last dependent. On a short-block partition, that tail write happens once per 1536-sample interval, in the last group. Frame-tail carriage needs at least one dependent. Its default is one dependent and strict signalling.

`--emdf-profile etsi` selects strict ETSI signalling. `--emdf-profile deployed` selects the deployed-compatible signalling. Either profile can be paired with either carriage. `--etsi-strict` remains available for native 5.1 and Flat 7.1. idx0, idx2 and idx4 already default to strict signalling.

```sh
mkdir aux-strict
mkdir aux-deployed
jocforge encode "$PWD/input.adm.wav" "$PWD/aux-strict/song.candidate.ec3" --joc-layout idx4 --dependent-count 2 --block-partition 3+3 --carriage auxdata --emdf-profile etsi
jocforge encode "$PWD/input.adm.wav" "$PWD/aux-deployed/song.candidate.ec3" --joc-layout idx4 --dependent-count 2 --block-partition 3+3 --carriage auxdata --emdf-profile deployed
```

## Legacy AC-3 core

`--carriage legacy-ac3` writes a fixed 1280 kbps elementary stream for idx0 through idx4. Each 1536-sample interval is one 640 kbps, 48 kHz AC-3 I0 followed by one 640 kbps E-AC-3 D0. Each substream is 2560 bytes. The mode is fixed at six blocks and one dependent. I0 is a 3/2 bed that can play on its own. D0 carries frame-tail JOC/OAMD and defaults to strict ETSI signalling. Add `--emdf-profile deployed` for the deployed-compatible profile. All four `--lfe-owner` values are available. The default LFE owner is `independent`.

D0 replaces L and C on idx0 and idx3. On idx1 it supplements the rear-surround pair (`0x0200`). On idx2 and idx4 it supplements the front height pair (`0x0010`). When D0 supplements or replaces LFE, the channel map also sets `0x0001`.

In an XML job, set D0 `data_rate` to 640. `--preflight-only` reports the total rate and the length of each access unit.

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/idx0-legacy.ec3" --joc-layout idx0 --carriage legacy-ac3 --total-rate 1280 --lfe-owner replace-independent
```

Seven-carrier output still uses its own empty directory:

```sh
mkdir height-core
jocforge encode "$PWD/input.adm.wav" "$PWD/height-core/song.candidate.ec3" --joc-layout idx4 --carriage legacy-ac3 --total-rate 1280 --lfe-owner dependent
```
