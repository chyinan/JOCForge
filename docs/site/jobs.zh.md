# XML 作业与响度

`jocforge encode --job-xml job.xml` 从作业文件读取输入和输出。命令行上只保留作业路径和其他选项。加上 `--preflight-only` 只检查 XML 和输入文件，不编码。这个选项也会打印剪辑前后每个声道的采样数。

`adm` 和 `atmos_mezz` 都读取 ADM BWF。解析后仍保留 XML 里声明的输入类型。默认编码是 5.1 EC-3。

编码 Flat 7.1 时加上 `--joc-layout flat7x`。这时 `data_rate` 是 D0 码率，`640` 表示总码率 1280 kbps。输出路径仍须是新空目录中的 `.candidate.ec3`。使用 CMAF 时，在命令行选择容器和总码率，把 `data_rate` 填为 640，并把 `.mp4` 文件名写在 `output/ec3` 下。详见[输出封装](containers.md)。

## 时间范围

在 `filter/audio/encode_to_atmos_ddp` 中可以截取 ADM，并加上首尾静音。下面的片段保留 00:01:00 至 00:01:02，前置 4 秒，后置 1 秒。节目长 7 秒。EC-3 按 32 ms 一帧写入，所以成品长 7.008 秒。

```xml
<time_base>file_position</time_base>
<timecode_frame_rate>25</timecode_frame_rate>
<start>00:01:00:00</start>
<end>00:01:02:00</end>
<prepend_silence_duration>4</prepend_silence_duration>
<append_silence_duration>1</append_silence_duration>
```

- `start` 和 `end` 可以省略，默认是文件开头和文件末尾。区间是左闭右开。也可以写成 `00:01:00.250` 这样的十进制秒。
- `HH:MM:SS:FF` 需要帧率。编码元素上的 `timecode_frame_rate` 优先，其次用输入端帧率。十进制秒时间码可以把帧率留为 `not_indicated`。结束位置如果是纯整数帧号，这一帧包含在区间内。
- 首尾静音以秒计，可以是小数。省略或留空都按 0 处理。响度测量、编码和对象位置使用同一条剪辑时间线。

## 音频元数据

Line 和 RF 的 DRC 互相独立。每一边可以是 `film_light`、`film_standard`、`music_standard`、`music_light` 或 `none`。默认组合保持编码器原有的输出。四个下混电平和优先下混模式会写入码流。

响度测量可以选用无门限的 BS.1770-1，有门限的 BS.1770-2、BS.1770-3、BS.1770-4，或 LeqA。当前 5.1 测量里，BS.1770-2、-3 和 -4 共用同一套门限算法。打开对白检测后，Earshot 标出语音。语音占比达到阈值时报告对白响度，低于阈值时报告节目响度。

环绕修剪和高度修剪互相独立。每一项都可以用 XML 里声明的档位。手动值按位置衰减。档位和手动值同时存在时，合成衰减不超过两项上限中较大的那一个。

## 响度缓存

第一次编码会完整测量 ADM。同一内容再次编码时，只要修剪、首尾静音和测量设置相同，就可以跨码率、跨 5.1 和 Flat 7.1 复用。命中时 CLI 打印 `Loudness cache hit`。测量算法或它的内置参数变了，会重新测量。

缓存保存响度、峰值和门控方式的精确数值，以及校验信息。Windows 默认目录是 `%LOCALAPPDATA%\JOCForge\adm-loudness-v1`。macOS 默认目录是 `~/Library/Caches/JOCForge/adm-loudness-v1`。`JOCFORGE_CACHE_DIR` 用来换掉这个根目录。删掉该目录会强制重新测量。目录不可写时，当前这次导出仍会完成。
