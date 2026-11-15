# User command timeline

记录影响全曲编码结果的用户指定尝试；每次编码使用独立目录。

| Attempt | Input | Result | Artifact directory |
| --- | --- | --- | --- |
| E-1093 | `D:\Love Song 方大同.wav` | Meter 完成；首个 AU 前因 ID14 inner payload 容量错误停止，无输出。 | `D:\JOCForge-attempts\E1093-love-song-flat7x-20261115-7346cf8` |
| E-1095 | `D:\Love Song 方大同.wav` | 发布 19 AU 后，ID14 sparse channel guard 拒绝 Flat7x index。 | `D:\JOCForge-attempts\E1095-love-song-flat7x-20261115-c508602` |
| E-1096 | `D:\Love Song 方大同.wav` | 发布 602 AU 后，C7 fit 搜索遇到不可量化 support candidate 并停止。OpenJOC 接纳已写出的 602 AU partial。 | `D:\JOCForge-attempts\E1096-love-song-flat7x-20261115-4672fb8` |
| E-1097 | `D:\Love Song 方大同.wav` | 发布 19 AU 后，parser 仍使用五声道 sparse initial-channel 上限。 | `D:\JOCForge-attempts\E1097-love-song-flat7x-20261115-2da6ed9` |
| E-1098 | `D:\Love Song 方大同.wav` | 完整 8,417 AU / 1,664 kbps candidate；OpenJOC observed-vendor compatibility 8,417/8,417，并渲染 7.1 float WAV。 | `D:\JOCForge-attempts\E1098-love-song-flat7x-20261115-3f86ce3` |
| E-1099 | `D:\Love Song 方大同.wav` | 2026-11-15 16:35:13 +08 启动，PID 27704；中途被外部 force stop，保留 3,217 AU / 21,412,352-byte partial。OpenJOC parsed/admitted 3,217/3,217，0 malformed；deployed compatibility pass，ETSI strict 仅剩 ID11/ID14 `codecdatae=0`。AU0 placement receipt 报告 block 2、bits 12201..15979、Codec generation 3、F15 label 2，两个 generation 属于不同 typed domain。 | `D:\JOCForge-attempts\E1099-love-song-flat7x-20261115-26d0454` |
| E-1100 | `D:\Love Song 方大同.wav` | 完整完成 8,417 AU / 56,023,552 B；OpenJOC observed-vendor compatibility 8,417/8,417，0 malformed；7.1 float WAV 已渲染。AU0 placement receipt：block 2、bits 12201..15979、AU 0、Codec source generation 3；F15 source label 2 为不同 typed domain。 | `D:\JOCForge-attempts\E1100-love-song-flat7x-20261115-26d0454` |

E-1098 和 E-1100 的 WAV 分别位于各自 attempt 的 `render\love-song-flat7x-openjoc-7.1.wav`。所有 candidate 均为实验输出，不表示与 DEE 等价。
