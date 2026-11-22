![JOCForge](assets/jocforge-logo.png){ .jocforge-logo }

# JOCForge

将 48 kHz ADM BWF 母版编码为 E-AC-3 JOC 音轨。独立 Rust 编码器读取声床、动态对象及其位置，写出音频载体、对象重建系数和空间元数据。

[快速开始](quick-start.md){ .md-button .md-button--primary }
[查看源码](https://github.com/chyinan/JOCForge/tree/main){ .md-button }

<div class="grid cards" markdown>

-   :material-speaker-multiple: **[布局与对象](layouts.md)**

    选择布局、码率和编码对象数。

-   :material-waveform: **[子流与信令](substreams.md)**

    配置 LFE 归属、块划分及 EMDF 承载位置。

-   :material-file-music: **[输出封装](containers.md)**

    生成 CMAF 分片或将 EC3 封装为 MKA、MP4。

-   :material-tune: **[XML 作业与响度](jobs.md)**

    设置渲染区间、首尾静音、DRC 及响度测量。

</div>
