# 布局与对象

| CLI 选项 | 重建载体 | 相位信令 | 默认总码率 |
|---|---|---|---:|
| `--joc-layout idx0` | 5.X | 关 | 1024 kbps |
| `--joc-layout idx1` 或 `flat7x` | Flat 7.X | 关 | 1664 kbps |
| `--joc-layout idx2` | 5.X+2 | 关 | 1664 kbps |
| `--joc-layout idx3` 或省略布局 | 5.X | 开 | 1024 kbps |
| `--joc-layout idx4` | 5.X+2 | 开 | 1664 kbps |

直接编码 idx0 至 idx4 的原生 EC-3 时，各布局支持其全部六档码率，可通过 `--joc-objects 8` 或 `--joc-objects 12` 试用较少的 JOC 对象。默认值为 15，LFE 单独计数。实验档在系数拟合前归并相邻对象槽，并同步写出对应的动态位置。较早版本的 OpenJOC 对实验档和默认档使用不同的 Stereo 渲染路径。

支持从节目起点开始、连续覆盖整段音频的笛卡尔对象自动化。每个对象可以包含多个 `audioBlockFormat`。编码器按每 1536 个采样（48 kHz 下为 32 ms）的控制回调读取对象位置，并根据 `jumpPosition` 和 `interpolationLength` 计算过渡。声床继续使用固定扬声器位置。

## 原生码率

CLI 将 48 kHz ADM BWF 编码为 Flat 7.1 E-AC-3 JOC。可选总码率为 **1024、1088、1216、1280、1408 和 1664 kbps**，默认 1664 kbps。每档都保留 640 kbps 的独立 AC-3 核心，其余码率分配给依赖 E-AC-3 子流。六档共用 Flat 7.X 重建配置（索引 1）和七路载体，默认 15 个 JOC 对象。

1664 kbps 档实测使用方大同《Love Song》（4 分 29 秒）。OpenJOC 0.20.2 在兼容模式下接纳了全部 8,417 个访问单元并完成 7.1 渲染。最终码流无损封装为 MKA 后，已实测可在 Adobe Audition 中打开，并可用 Windows 媒体播放器播放。

JOCForge 的命令行编码器支持将 48 kHz ADM BWF 编码为带 5.1 声道核心的 E-AC-3 JOC 码流。可选码率为 384、448、576、640、768 和 1024 kbps。六档码率均已在本地完成整文件编码，并通过输入配置、帧长和无密钥完整性检查。

由于涉及 HMAC 私钥，iOS 系统解码器无法解码 JOCForge 生成的 5.1 全景声音轨，播放时为静音，目前本开源项目无法解决这一限制。用户自行修改本项目所产生的后果，由用户自行承担。

## 编码 Flat 7.1

把输入路径换成自己的 ADM BWF。Flat7x 要求输出放进新的空目录，文件名使用 `.candidate.ec3` 后缀。

```sh
mkdir flat7x
jocforge encode "$PWD/input.adm.wav" "$PWD/flat7x/output.candidate.ec3" --joc-layout flat7x --total-rate 1280
```

## 5.1 严格 ETSI 信令

`--etsi-strict` 为独立 5.1 EC-3 的 JOC 和 OAMD 写入严格 EMDF 描述符，并将 OAMD 的坐标变换模式设为无变换。省略该选项使用原有部署兼容格式。文件输入与 XML 作业都支持此选项。

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/output.ec3" --data-rate 1024 --etsi-strict
jocforge encode --job-xml job.xml --etsi-strict
```

## Flat 7.1 严格信令与 5.X+2 高度载体

Flat 7.1 可选严格 ETSI 信令。idx2 使用两个前方高度载体 Tfl/Tfr，默认启用严格信令。两者均使用 640 kbps AC-3 兼容核心和 E-AC-3 依赖子流，默认总码率为 1664 kbps。每次输出使用单独的空目录。

```sh
mkdir flat-strict
mkdir height-pair
jocforge encode "$PWD/input.adm.wav" "$PWD/flat-strict/song.candidate.ec3" --joc-layout flat7x --etsi-strict
jocforge encode "$PWD/input.adm.wav" "$PWD/height-pair/song.candidate.ec3" --joc-layout idx2
```

idx2 的七个重建输入为 L、R、C、Ls、Rs、Tfl、Tfr。编码器将高度床声道送入高度对，按动态对象坐标在地面和高度载体之间分配能量，再用这七路音频拟合 JOC 系数。依赖子流的通道映射为 `0x1810`，携带 Ls、Rs 和高度对；LFE 由兼容核心提供。

idx0 和 idx4 默认使用严格 ETSI 信令。idx0 使用单独的 E-AC-3 I0，可直接指定输出文件；idx4 与 idx2 共用 Tfl/Tfr 高度载体、对象位置分配和系数拟合，以 profile 索引区分相位信令。两者沿用同一套音频处理。配对子流的总码率使用 `--total-rate`，单流码率使用 `--data-rate`。

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/music-idx0.ec3" --joc-layout idx0
mkdir height-phase
jocforge encode "$PWD/input.adm.wav" "$PWD/height-phase/music.candidate.ec3" --joc-layout idx4
```
