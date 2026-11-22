# XML 作业与响度

除直接指定输入和输出文件外，也可以用 `jocforge encode --job-xml <job.xml>` 读取作业配置。当前支持 ADM BWF 输入和 EC-3 裸码流输出。`adm` 和 `atmos_mezz` 两种输入配置共用 BWF 读取器，解析后保留各自的输入类型。添加 `--preflight-only` 可只检查 XML 和输入文件。MP4 封装可在编码后按上文示例使用 FFmpeg 完成。使用 CMAF 时，通过 CLI 选择容器和总码率，在 XML 中指定 D0 码率与输出路径，详见[输出封装](containers.md)。

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

## 响度缓存

JOCForge 为当前用户保存响度缓存。第一次编码时完整测量 ADM；再次编码同一内容时，只要实际裁剪位置、首尾静音和测量设置相同，就可以跨码率、跨 5.1 和 Flat 7.1 复用。命中时 CLI 会显示 `Loudness cache hit`。测量算法版本或内置参数变化会重新测量。

缓存保存响度、峰值、门控方式等完整测量结果的精确数值及校验信息。Windows 默认目录为 `%LOCALAPPDATA%\JOCForge\adm-loudness-v1`，macOS 为 `~/Library/Caches/JOCForge/adm-loudness-v1`。可通过 `JOCFORGE_CACHE_DIR` 指定缓存根目录，或删除缓存以强制重新测量。目录不可写时，编码器照常完成当前导出。
