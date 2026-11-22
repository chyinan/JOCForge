# 输出封装

## CMAF 分片输出

`--container cmaf` 输出单文件分片 MP4。文件以 `ftyp + moov` 初始化，每个 `moof + mdat` 装一个完整的 1536 采样访问单元。五种 JOC profile 均使用严格 EMDF、六块 E-AC-3 I0+D0。默认无 LFE，指定 `--lfe-owner independent` 时由 I0 携带 LFE，指定 `--lfe-owner dependent` 时由 D0 补充普通 LFE，指定 `--lfe-owner replace-independent` 时 D0 替换 I0 的同一 LFE，D0 帧尾携带 JOC/OAMD。

```powershell
jocforge encode "$PWD/input.adm.wav" "$PWD/height-phase.mp4" --container cmaf --joc-layout idx4 --total-rate 1664
```

CMAF 默认总码率 1664 kbps，可选 1024、1280、1408、1536、1664。D0 固定 640 kbps，其余码率分给 I0。显式指定的 LFE、依赖子流数、块划分、承载和信令必须符合上述条件。XML 作业仍通过 `output/ec3` 指定输出路径，文件名可填 `.mp4`，XML 的 `data_rate` 填 640；在 CLI 上选择 CMAF 容器和总码率。

## 无损封装为 MKA 或 MP4

沿用[布局与对象](layouts.md)中 Flat 7.1 示例的 `$ec3` 和 `$out`。`-c:a copy` 直接复制 EC3 音轨并保留 JOC 元数据。

**MKA**（已实测可在 Audition 和 Windows 媒体播放器中打开）

```powershell
ffmpeg -n -i "$ec3" -map 0:a:0 -c:a copy -f matroska (Join-Path $out "output.mka")
```

![Adobe Audition 中打开的 MKA 编码结果，八条轨道依次为 L、R、C、LFE、Ls、Rs、Lsd、Rsd](assets/screenshot.png)

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
