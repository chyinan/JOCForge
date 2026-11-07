# CHNA 到 AXML physical slot 覆盖边界

对最短 36 轨样本和唯一 38 轨样本，各自的 fmt channel 数、CHNA numTracks/UID 数与 capacity 一致。CHNA 的 1-based trackIndex 在 1..N 内无缺失、无重复；每个 CHNA UID 在 AXML 有且仅有一个 audioTrackUID 定义，trackRef/packRef 相符。每条 trackFormat/streamFormat 路径解析到一个唯一直接 audioChannelFormatID，且覆盖该输入的全部直接 channel-format 集合。

这证明两份输入自身的 metadata row/UID 映射完整，不证明 DEE 的 U individual/group containers 或 K rows 与 CHNA slots 同代一一对应。它也不证明 DEE/JOCForge 已接纳任意轨数。packRef 的多成员关系仍作为 pack grouping 保留，不拿来推导逐轨 channel 一对一关系。

只读取 RIFF/WAVE header、fmt、CHNA 和 AXML；data chunk 被 seek 跳过，PCM 读取 0 字节。未算 whole-file hash、未复制或改名输入、未运行 DEE。
