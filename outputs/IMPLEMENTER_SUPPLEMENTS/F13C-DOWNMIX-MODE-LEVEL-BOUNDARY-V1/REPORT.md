# Downmix mode 与 mix-level 输入矩阵

## 可实施的 source-path 规则

当前唯一闭合的消费者是 JOCForge 选定 BSI source path。该路径只接受 `preferred_downmix_mode=loro`，四个 mix-level 都必须是 `-3`。它把这些输入映射为 BSI `dmixmod=2` 和四个 level code `4`，并交给选定 frame writer 写入。

这是源码路径证据。它只覆盖 38 声道、48 kHz、PCM24、1024 kbps 的选定 profile。它没有证明 attempt-10 实际输出了这些位，也没有证明 DEE 的音频效果。36 声道和其他模式仍未绑定到该 source path。

## 输入值与消费者边界

DDP XML 的 `preferred_downmix_mode` schema/parser 静态接受四个 token：`loro`、`ltrt`、`ltrt-pl2` 和 `not_indicated`，并映射到 parser code `0`～`3`。这些 code 未绑定到有效存储值或选定处理消费者。DME UI 只提供 `Stereo→loro`、`Surround→ltrt` 和 `Not Indicated→not_indicated`，且只序列化 active 值。`ltrt-pl2` 没有对应 UI 选项。

DME host state 到 `clientJob` JSON 有静态映射，但没有找到它写入指定 DDP XML leaf 的路径。DME UI 与该 XML 属性之间的连接仍为 `UNBOUND`。

四个 level 控件各有独立的字符串值和 active flag。DME 的 host key 与 native XML key 不同。native parser 写入的 helper-result slot 尚未绑定到 typed DDP descriptor、BSI consumer 或音频 consumer。不要把 parser 接受字符串当成效果支持。Native `ddp_dmixmod` validator 使用另一组 token，和 `preferred_downmix_mode` 的映射仍为 `UNBOUND`。

## 最小 input-only 矩阵

矩阵先用四个 mode token 和共同基准 `-3` 做 mode sweep，再分别把四个 level 从 `-3` 改为 `-1.5`。这两个有限值都出现在四个控件的可见 UI 值域中。矩阵是候选输入，不是结果；所有行均为 `NOT_RUN`。

现有 `loro`／`ltrt` XML pair 只通过了 schema 预检，encoder 实验仍为 `NOT_RUN`。`ltrt-pl2` 和 `not_indicated` 没有准备对应 job XML。四个 level 单字段变体目前只在已有 A/B 计划中，未生成 job XML。

JOCForge 规则来自已封存的 source-path 包。当前研究 checkout 不含该包记录的 `crates/` 文件，因此本次没有重算当前实现文件哈希。

## 36／38 声道覆盖

复用的 metadata-only scan 记录 7 个 ADM：36 声道 6 个，38 声道 1 个，采样率均为 48 kHz、位深为 24 bit。这些统计不绑定具体实验输入身份，也不证明输出效果。

现有 38 声道 `loro`／`ltrt` 预检复用了用户确认的输入身份，但本次没有重算输入哈希。36 声道没有选定样本身份。两种 profile 的验证向量都保持 `NOT_RUN`。

## 实验决定

A/B 能区分可观察输出是否随输入变化，但当前不能把变化绑定到 typed consumer。已有 R4 记录还表明，WinDbg launch 接口要求完整命令行，其中 license 参数值在当前任务上下文不可用；不得填入占位值。已有 `loro`／`ltrt` pair 未运行，本包也未启动 DEE。

## 范围

本包复用已封存的 Ghidra 12.2、DME host、JOCForge source-path、输入 metadata 和 A/B preflight receipts，没有重扫已闭合边。未运行 DEE、WinDbg 或 A/B；未读媒体、PCM、压缩 payload、key 或 license；未联系实现。`UNBOUND` 和 `NOT_RUN` 均按原证据保留。
