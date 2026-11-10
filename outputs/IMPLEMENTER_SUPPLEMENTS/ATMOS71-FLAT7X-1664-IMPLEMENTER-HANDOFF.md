# 唯一 sanitized implementation handoff 条目

Wash 封存后，只需将以下一个合同条目交给实现侧：

> 在 JOCForge-isolated 中新增与现有 native 5.1 多码率 row 并列的 7.1 flat7x profile。目标 AU 为 48 kHz、6 blocks、I0=640 kbps/2,560 bytes 后接 D0=1,024 kbps/4,096 bytes，合计 1,664 kbps/6,656 bytes；每个 AU 以同一 source generation 绑定两帧，并原子发布 `[I0,D0]`。I0 保持 independent、ID 0、acmod=7、lfeon=1；D0 使用 dependent、ID 0、acmod=5、lfeon=0、chanmap=0x1A00，对应的 coded locations 为 Ls/Rs/Lrs/Rrs。flat7x 使用 7 个非 LFE reconstruction carriers（L/R/C/Ls/Rs/Lrs/Rrs）、15 个 coded objects/rows 和 complexity index 16。保留 7 个 production profile 语义维度；`JocDownmixProfile` 与 `JocReconstructionLayout::Flat7x` 分型，flat7x 通过 `joc_dmx_config_idx=1` 表示。目标要求每个 D0 block 2 的单个 skipfld 携带完整 EMDF 容器，容器含 prefix 总长 `C<=511`；超限时发布整个 AU 前失败。共用现有 I0、bit I/O、CRC、metadata 与 publication seam 时，以类型和 source generation 明确绑定。保留 5.1 路径的现有输出身份。D0 audio algorithm、ADM role join、7.1 downmix mode/phase 与 native writer owner 均保持 `UNBOUND`；实现进度和输出验收须按这些门槛报告，当前目标整体 `production_allowed=false`。

此条目承载用户目标与 Wash 审核后的架构决定。它不携带 DEE 反汇编、候选地址、媒体或 payload，也不把 unknown 写成已验证能力。其余材料留在 Wash review 包。
