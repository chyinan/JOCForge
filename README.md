# JOCForge

![JOCForge CLI：encode / completions 子命令与 --version 输出](docs/assets/cli-help.png)

## Flat 7.1 编码

CLI 将 48 kHz ADM BWF 编码为 Flat 7.1 E-AC-3 JOC。可选总码率为 **1024、1088、1216、1280、1408 和 1664 kbps**，默认 1664 kbps。每档都保留 640 kbps 的独立 AC-3 核心，其余码率分配给依赖 E-AC-3 子流。六档共用 Flat 7.X 重建配置（索引 1）、七路载体和 15 个 JOC 对象。

1664 kbps 档实测使用方大同《Love Song》（4 分 29 秒）。OpenJOC 0.20.2 在兼容模式下接纳了全部 8,417 个访问单元并完成 7.1 渲染。最终码流无损封装为 MKA 后，已实测可在 Adobe Audition 中打开，并可用 Windows 媒体播放器播放。

### 从 ADM 编码到 EC3

支持从节目起点开始、连续覆盖整段音频的笛卡尔对象自动化。每个对象可以包含多个 `audioBlockFormat`。编码器按每 1536 个采样（48 kHz 下为 32 ms）的控制回调读取对象位置，并根据 `jumpPosition` 和 `interpolationLength` 计算过渡。声床继续使用固定扬声器位置。

在仓库根目录运行以下 PowerShell 命令，将输入路径换成自己的 ADM BWF。需要 Rust 工具链和 FFmpeg。Flat7x CLI 要求输出到新的空目录，并使用 `.candidate.ec3` 后缀。

```powershell
cargo build --release --locked -p jocforge
if ($LASTEXITCODE -ne 0) { throw "构建失败" }

$adm = (Resolve-Path -LiteralPath "D:\Audio\input.adm.wav").Path
$out = (New-Item -ItemType Directory -Path (Join-Path $PWD ("flat7x-" + [guid]::NewGuid()))).FullName
$ec3 = Join-Path $out "output.candidate.ec3"

.\target\release\jocforge.exe encode "$adm" "$ec3" --joc-layout flat7x --total-rate 1280
if ($LASTEXITCODE -ne 0) { throw "Flat 7.1 编码失败" }
```

### 5.1 严格 ETSI 信令

`--etsi-strict` 为独立 5.1 EC-3 的 JOC 和 OAMD 写入严格 EMDF 描述符，并将 OAMD 的坐标变换模式设为无变换。省略该选项使用原有部署兼容格式。文件输入与 XML 作业都支持此选项。

```powershell
jocforge encode input.adm.wav output.ec3 --data-rate 1024 --etsi-strict
jocforge encode --job-xml job.xml --etsi-strict
```

### Flat 7.1 严格信令与 5.X+2 高度载体

Flat 7.1 可选严格 ETSI 信令。idx2 使用两个前方高度载体 Tfl/Tfr，默认启用严格信令。两者均使用 640 kbps AC-3 兼容核心和 E-AC-3 依赖子流，默认总码率为 1664 kbps。每次输出使用单独的空目录。

```powershell
mkdir flat-strict, height-pair
jocforge encode input.adm.wav flat-strict/song.candidate.ec3 --joc-layout flat7x --etsi-strict
jocforge encode input.adm.wav height-pair/song.candidate.ec3 --joc-layout idx2
```

idx2 的七个重建输入为 L、R、C、Ls、Rs、Tfl、Tfr。编码器将高度床声道送入高度对，按动态对象坐标在地面和高度载体之间分配能量，再用这七路音频拟合 JOC 系数。依赖子流的通道映射为 `0x1810`，携带 Ls、Rs 和高度对；LFE 由兼容核心提供。

### 五种 JOC profile

| CLI 选项 | 重建载体 | 相位信令 | 默认总码率 |
|---|---|---|---:|
| `--joc-layout idx0` | 5.X | 关 | 1024 kbps |
| `--joc-layout idx1` 或 `flat7x` | Flat 7.X | 关 | 1664 kbps |
| `--joc-layout idx2` | 5.X+2 | 关 | 1664 kbps |
| `--joc-layout idx3` 或省略布局 | 5.X | 开 | 1024 kbps |
| `--joc-layout idx4` | 5.X+2 | 开 | 1664 kbps |

idx0 和 idx4 默认使用严格 ETSI 信令。idx0 使用单独的 E-AC-3 I0，可直接指定输出文件；idx4 与 idx2 共用 Tfl/Tfr 高度载体、对象位置分配和系数拟合，以 profile 索引区分相位信令。两者沿用同一套音频处理。配对子流的总码率使用 `--total-rate`，单流码率使用 `--data-rate`。

```powershell
jocforge encode input.adm.wav music-idx0.ec3 --joc-layout idx0
mkdir height-phase
jocforge encode input.adm.wav height-phase/music.candidate.ec3 --joc-layout idx4
```

### 多依赖子流与 LFE 归属

`--dependent-count N` 指定每个 1536 采样访问单元内的依赖子流数。idx0、idx3 可选 0–5，idx1、idx2、idx4 可选 1–6。显式指定流数或 LFE 归属时，JOCForge 使用原生 E-AC-3 I0 和连续编号的 D0…Dn，每帧 6 个音频块。最后一个依赖子流承载严格 ETSI JOC/OAMD，零依赖时由 I0 承载。节目总码率在这些子流之间分配。常规默认输出继续使用原来的编码路径。

| `--lfe-owner` | I0 | D0 | 解码后的 LFE |
|---|---|---|---|
| `independent`（默认） | 有 | 无 | 取 I0 |
| `dependent` | 无 | 有 | D0 补充 |
| `replace-independent` | 有 | 有 | D0 替换 I0 |
| `none` | 无 | 无 | 无 |

单独指定 LFE 时，五载体布局默认使用零个依赖子流，七载体布局默认使用一个。需要 D0 承载 LFE 时至少使用一个。依赖声道映射按 profile 的规范组顺序分配，后环绕对或高度对保持成对。文件输入和 XML 作业使用相同选项；七载体布局的输出仍使用独立空目录中的 `.candidate.ec3`。

```powershell
# 5.X：五个依赖子流，D0 的 LFE 替换 I0，D4 承载 JOC。
jocforge encode input.adm.wav mix.ec3 --joc-layout idx0 --dependent-count 5 --lfe-owner replace-independent
# 5.X+2：六个依赖子流，D0 补充 LFE，D5 承载高度对和 JOC。
mkdir height-multi
jocforge encode input.adm.wav height-multi/mix.candidate.ec3 --joc-layout idx4 --total-rate 1664 --dependent-count 6 --lfe-owner dependent
```

### 短块划分

`--block-partition` 支持 `3+3`、`2+2+2` 和 `1x6`（也可写作 `1+1+1+1+1+1`）。默认值是 `6`。短块使用原生 E-AC-3，每个 1536 采样区间的总码率与时长按原参数计算。每个子区间先写 I0，随即写其 D0…Dn；同一组的块数一致。最后一个子区间的最后一条子流承载一次 JOC/OAMD。过低码率若容不下全部同步帧头、指数和元数据，会返回容量错误。

文件输入和 XML 作业共用该选项，可与 `--dependent-count`、`--lfe-owner` 配合。例如：

```powershell
mkdir height-short
jocforge encode input.adm.wav height-short/song.candidate.ec3 --joc-layout idx4 --total-rate 1664 --block-partition 3+3 --dependent-count 2 --lfe-owner dependent
```

### EMDF 承载位置与信令

`--carriage auxdata`（主分支拼写为 `etsi`）将受保护的 EMDF 容器写入最后一个依赖子流的帧尾。短块划分时，每个 1536 采样区间只在最后一组写入一次。默认使用一个依赖子流；显式指定流数时，帧尾模式要求至少一个。音频块内的承载使用 `--carriage skipfld`。

`--emdf-profile etsi` 和 `--emdf-profile deployed` 分别选择严格 ETSI 信令和部署兼容信令，两种承载都可以搭配任一信令。原有 `--etsi-strict` 仍可用。单独指定帧尾承载时，默认使用严格信令。文件输入和 XML 作业共用这些参数。

```powershell
mkdir aux-strict, aux-deployed
jocforge encode input.adm.wav aux-strict/song.candidate.ec3 --joc-layout idx4 --dependent-count 2 --block-partition 3+3 --carriage auxdata --emdf-profile etsi
jocforge encode input.adm.wav aux-deployed/song.candidate.ec3 --joc-layout idx4 --dependent-count 2 --block-partition 3+3 --carriage auxdata --emdf-profile deployed
```

### 无损封装为 MKA 或 MP4

在同一个 PowerShell 会话中任选一种容器。`-c:a copy` 直接复制 EC3 音轨并保留 JOC 元数据。

**MKA**（已实测可在 Audition 和 Windows 媒体播放器中打开）

```powershell
ffmpeg -n -i "$ec3" -map 0:a:0 -c:a copy -f matroska (Join-Path $out "output.mka")
```

![Adobe Audition 中打开的 MKA 编码结果，八条轨道依次为 L、R、C、LFE、Ls、Rs、Lsd、Rsd](docs/assets/screenshot.png)

上图是《Love Song》编码结果封装为 MKA 后在 Adobe Audition 中的波形与频谱视图。八条轨道依次为 L、R、C、LFE、Ls、Rs、Lsd、Rsd，时间轴总长 4 分 29 秒。

**MP4**（可选用 `.m4a` 后缀）

```powershell
$mp4 = Join-Path $out "output.mp4"
ffmpeg -n -i "$ec3" -map 0:a:0 -c:a copy -f mp4 "$mp4"
if ($LASTEXITCODE -ne 0) { throw "MP4 封装失败" }
```

需要 `.m4a` 文件时，再运行下面的命令。FFmpeg 根据 `.m4a` 后缀默认选择 `ipod` 封装器；先生成 MP4 再改名，可以保留 MP4 封装和其中的 `ec-3` 音轨。

```powershell
Rename-Item -LiteralPath $mp4 -NewName "output.m4a"
```

生成的 `.m4a` 已实测可由 iOS 系统解码器打开。

Flat7x 在 CLI 中仍标为实验性。六档均已用一首含动态对象的 48 kHz 达芬奇 ADM 母版完成整曲编码、FFmpeg 解码和 OpenJOC 7.1 渲染。OpenJOC 的部署兼容检查通过，ETSI 严格检查仍报告载荷 11 和 14 的 `codecdatae` 标志差异。

## 5.1 编码

由于涉及 HMAC 私钥，iOS 系统解码器无法解码 JOCForge 生成的 5.1 全景声音轨，播放时为静音，目前本开源项目无法解决这一限制。用户自行修改本项目所产生的后果，由用户自行承担。

JOCForge 的命令行编码器支持将 48 kHz ADM BWF 编码为带 5.1 声道核心的 E-AC-3 JOC 码流。可选码率为 384、448、576、640、768 和 1024 kbps。六档码率均已在本地完成整文件编码，并通过输入配置、帧长和无密钥完整性检查。

下面的 PowerShell 命令使用上文构建的 Release 编码器，以 1024 kbps 输出 5.1 码流。

```powershell
.\target\release\jocforge.exe encode "D:\input.adm.wav" "D:\output.ec3" --data-rate 1024
```

### 使用 XML 作业文件

除直接指定输入和输出文件外，也可以用 `jocforge encode --job-xml <job.xml>` 读取作业配置。当前支持 ADM BWF 输入和 EC-3 裸码流输出。`adm` 和 `atmos_mezz` 两种输入配置共用 BWF 读取器，解析后保留各自的输入类型。添加 `--preflight-only` 可只检查 XML 和输入文件。MP4 封装可在编码后按上文示例使用 FFmpeg 完成。CLI 的 XML 作业不支持直接输出 MP4 或 DASH。

Line 与 RF 可独立设置为 `film_light`、`film_standard`、`music_standard`、`music_light` 或 `none`。默认组合保留原有输出。下混的四个混合电平与优先模式写入码流。响度测量提供 BS.1770-1（无门限）以及 BS.1770-2、-3、-4（当前 5.1 测量布局共用门限算法），另可选 LeqA。启用对白检测时使用 Earshot 识别语音，语音占比达到阈值后采用对白响度，否则采用节目响度。5.1 环绕和高度修剪可独立选用 XML 声明的档位，手动值按位置衰减，组合衰减以上限较大的一项为限。

在 `filter/audio/encode_to_atmos_ddp` 中可以按文件位置截取 ADM，再添加片头和片尾静音。以下片段截取第 60 到 62 秒，前置 4 秒静音，后置 1 秒静音。节目长 7 秒。EC3 按 32 ms 整帧写入，本例成品长 7.008 秒。

```xml
<time_base>file_position</time_base>
<timecode_frame_rate>25</timecode_frame_rate>
<start>00:01:00:00</start>
<end>00:01:02:00</end>
<prepend_silence_duration>4</prepend_silence_duration>
<append_silence_duration>1</append_silence_duration>
```

- `start` 和 `end` 可以单独设置，分别默认文件开头和文件末尾。时间码表示左闭右开区间，也可以写为 `00:01:00.250` 这样的十进制秒时间码。
- 使用 `HH:MM:SS:FF` 时需提供帧率，编码端的 `timecode_frame_rate` 优先，其次使用输入端帧率。使用十进制秒时间码时可保留 `not_indicated`。纯整数帧号的结束帧包含在区间内。
- 首尾静音均以秒计，支持小数，省略或留空均按零处理。响度测量、音频编码和动态对象位置使用同一条剪辑时间线。添加 `--preflight-only` 可查看原文件和剪辑后的每声道采样数。
- XML 作业默认编码 5.1。使用 `jocforge encode --job-xml job.xml --joc-layout flat7x` 可编码 Flat 7.1。此时 XML 的 `data_rate` 指 D0 码率，例如 `640` 对应 1280 kbps 总码率，输出路径遵循上文的 Flat7x 文件命名要求。

### 响度缓存

CLI 和 NERX Studio 共用当前用户的响度缓存。第一次编码时完整测量 ADM；再次编码同一内容时，只要实际裁剪位置、首尾静音和测量设置相同，就可以跨码率、跨 5.1 和 Flat 7.1 复用。命中时 CLI 会显示 `Loudness cache hit`。测量算法版本或内置参数变化会重新测量。

缓存保存响度、峰值、门控方式等完整测量结果的精确数值及校验信息。Windows 默认目录为 `%LOCALAPPDATA%\JOCForge\adm-loudness-v1`，macOS 为 `~/Library/Caches/JOCForge/adm-loudness-v1`。可通过 `JOCFORGE_CACHE_DIR` 指定缓存根目录，或删除缓存以强制重新测量。目录不可写时，编码器照常完成当前导出。

### 开发与检查

Windows x64 Release 在 Ryzen 7 9800X3D 上编码一份 191.69 秒、26 声道的 48 kHz 动态 ADM。5.1 / 1024 kbps 首次导出用时 41.4 秒，其中响度测量 16.2 秒；旧版分别为 52.0 秒和 20.0 秒。复用响度时，5.1 / 1024 kbps 用时 25.2 秒，Flat 7.1 / 1664 kbps 用时 37.8 秒。两份整曲 EC3 和首次测量得到的响度、真峰值与门控结果均与旧版逐位一致。

项目采用 Rust 2024 工作区，包含领域逻辑、应用、文件读写、命令行和独立验证五个模块。编码请使用 CLI，应用层的 `encode()` 接口目前返回 `SourcePipelineUnavailable`。命令参数和版本可通过 `jocforge --help` 和 `jocforge --version` 查看。[架构说明](docs/architecture/ARCHITECTURE.md)

```powershell
rtk cargo fmt --all -- --check
rtk cargo check --workspace --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk proxy python scripts/check-boundaries.py
```
