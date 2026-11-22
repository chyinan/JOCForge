# 布局与对象

`--joc-layout` 选择重建载体，并决定相位信令是否打开。省略时使用 idx3。

| 布局 | 重建载体 | 相位信令 | 默认码率 |
|---|---|---|---|
| `idx0` | 5.X | 关 | 1024 kbps（`--data-rate`） |
| `idx1`、`flat7x` | Flat 7.X，后环绕 | 关 | 1664 kbps（`--total-rate`） |
| `idx2` | 5.X+2，左前高度和右前高度 | 关 | 1664 kbps（`--total-rate`） |
| `idx3`（默认） | 5.X | 开 | 1024 kbps（`--data-rate`） |
| `idx4` | 5.X+2，左前高度和右前高度 | 开 | 1664 kbps（`--total-rate`） |

五载体布局的 `--data-rate` 可以是 384、448、576、640、768 和 1024 kbps。七载体布局的 `--total-rate` 可以是 1024、1088、1216、1280、1408 和 1664 kbps。七载体总码率包含 640 kbps 的 AC-3 核心，其余分给 E-AC-3 依赖子流。

idx2 和 idx4 使用同一组七路输入：L、R、C、Ls、Rs、Tfl 和 Tfr。高度床声道进入高度对。对象能量按坐标分到地面载体和高度对，再用这七路音频拟合 JOC 系数。依赖子流的通道映射是 `0x1810`，携带 Ls、Rs 和高度对。LFE 由兼容核心提供。两个布局只以相位信令索引区分。

idx0、idx2 和 idx4 默认使用严格 ETSI 信令。`--etsi-strict` 也可以强制 5.1 和 Flat 7.1 使用这套信令。承载位置和 EMDF 档位见[子流与信令](substreams.md)。

## 对象

JOCForge 每 1536 个采样读取一次笛卡尔坐标。48 kHz 下，这个间隔是 32 ms。每个对象可以有多个 `audioBlockFormat`。过渡按 `jumpPosition` 和 `interpolationLength` 计算。声床保持固定扬声器位置。

原生 EC-3 的 `--joc-objects` 可以取 `8` 或 `12`，五种布局的每一档码率都可以用。默认是 15。LFE 不计入这个数量。较少对象的模式会先归并相邻对象槽，再拟合系数，并写出归并后的位置。较早的 OpenJOC 对较少对象和默认 15 对象使用不同的立体声渲染路径。

## 示例

路径必须是绝对路径。`$PWD` 在 PowerShell 和 POSIX shell 里都可以用。七载体裸码流必须放进新的空目录，文件名使用 `.candidate.ec3` 后缀。idx0 可以直接指定输出文件。

1280 kbps 的 Flat 7.1：

```sh
mkdir flat7x
jocforge encode "$PWD/input.adm.wav" "$PWD/flat7x/mix.candidate.ec3" --joc-layout flat7x --total-rate 1280
```

关闭相位的 5.X：

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/music-idx0.ec3" --joc-layout idx0
```

高度对，分别关闭和打开相位：

```sh
mkdir height-pair
mkdir height-phase
jocforge encode "$PWD/input.adm.wav" "$PWD/height-pair/song.candidate.ec3" --joc-layout idx2
jocforge encode "$PWD/input.adm.wav" "$PWD/height-phase/music.candidate.ec3" --joc-layout idx4
```

## 播放

JOCForge 的 5.1 输出在实测的 iOS 系统解码器上为静音，原因是缺少 HMAC 密钥。本仓库不包含这把密钥，解码器完不成校验。用户自行更改本开源仓库的后果，由用户自行承担。

Flat 7.1 仍是实验档。4 分 29 秒的《Love Song》以 1664 kbps 编码后得到 8,417 个访问单元。OpenJOC 0.20.2 在兼容模式下接纳了全部访问单元，并完成 7.1 渲染。另一项实测用的是一首 48 kHz 达芬奇 ADM 母版，节目里有动态对象。六档 Flat 7.1 都完成了整曲编码、FFmpeg 解码和 OpenJOC 7.1 渲染。部署兼容检查通过。严格 ETSI 检查仍报告载荷 11 和载荷 14 的 `codecdatae` 标志差异。

封装后的播放结果见[输出封装](containers.md)。
