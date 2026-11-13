# 官方演示样本：前两个 AU 的 D0 BSI 稳定性

## 已观察

在用户指定的官方演示 MKA 的前两个固定长度 packet 中，I0 在前、D0 在后；两个 D0 都是 2,560-byte、640-kbps 样本帧。两帧从 syncword 到 chanmap 的 14 个字段值一致，`chanmap=0x1A00`。字段值、宽度和 frame-relative bit 范围见 `FIELD-VALUES.json`。

探针只读取 22 个源字节：SimpleBlock 的 5-byte lacing header、第二 AU 的 7-byte I0 header，以及第二 AU D0 的 10-byte header/BSI。记录只保存字段数值、位置和状态，没有保存 frame、packet 或 payload 字节。

## 使用范围

这是同一官方样本前两个 AU 的 BSI 比较向量，适合结构核验。`frmsiz=1279`、`dialnorm=31`、`compre=1`、`compr=1` 是这个 640-kbps 样本的观察值；它们不是 1024-kbps Flat7x 通用编码默认值。目标 D0 1024 kbps / 4096 bytes / `frmsiz=2047` 来自独立目标合同。该探针在 BSI `chanmap` 结束处停止，未解析 skipfld 或 audio blocks。

零起始 D0 `block_index=2` 的编号含义另见 `FLAT7X-D0-BLOCK-INDEX-CONVENTION-WASH-V1`。

此补充只提供样本向量，不定义 D0 音频生成、ADM 行映射、EMDF 长度算法或码率行为。
