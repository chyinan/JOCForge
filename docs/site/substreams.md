# Substreams and signalling

I0 carries the independent presentation. D0 and subsequent dependent substreams can supplement or replace channels in that presentation.

| Option | Values | Effect |
|---|---|---|
| `--dependent-count` | 0–5 for idx0/3, 1–6 for idx1/2/4 | Choose the number of dependent substreams. Rear and height pairs stay together. |
| `--lfe-owner` | `independent`, `dependent`, `replace-independent`, `none` | Use I0 LFE, supplement via D0, replace I0 LFE with D0, or omit LFE. D0 ownership needs a dependent. |
| `--block-partition` | `6` (default), `3+3`, `2+2+2`, `1x6` | Divide a 1536-sample interval into equal groups. Each starts with I0. |
| `--carriage` | `skipfld`, `auxdata` (alias `etsi`), `legacy-ac3` | Place EMDF in an audio block or at the frame tail, or use a legacy core. |
| `--emdf-profile` | `etsi`, `deployed` | Select strict or deployed-compatible signalling independently of carriage. |

Frame-tail carriage requires a dependent. The last dependent of the last group carries JOC/OAMD once per interval. A capacity error indicates that the selected bitrate cannot fit the syncframe headers, exponents and metadata. Use `--etsi-strict` for strict signalling on native 5.1 or Flat 7.1. Native idx0, idx2 and idx4 default to strict signalling.

## Legacy AC-3 core

All five profiles support a 1280 kbps elementary stream with six blocks, one 640 kbps AC-3 I0 and one 640 kbps E-AC-3 D0. Frame-tail JOC/OAMD lives in D0. Both signalling modes and all four LFE assignments are available. In XML jobs set D0 `data_rate` to 640.

```powershell
jocforge encode "D:/Audio/input.adm.wav" "D:/Audio/core.ec3" --joc-layout idx0 --carriage legacy-ac3 --total-rate 1280 --lfe-owner replace-independent
```
