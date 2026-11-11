---
status: implementer-ready-limited-header-segment
topic: "Flat7x I0 legacy AC-3 header segment"
spec: "ETSI TS 102 366 V1.4.1, clauses 4.3.1, 4.3.2 and Table 4.13"
audience: implementer
production_allowed: false
tags: [JOCForge, Flat7x, I0, AC-3, header]
---

# Flat7x I0 AC-3 header segment

## 已洗白的目标

官方演示结构包记录 I0 为 legacy AC-3 core：bsid=6、acmod=7、lfeon=1、48 kHz、6 blocks、每声道 1,536 samples、每帧 2,560 bytes。Flat7x 1664 profile contract 将目标 I0 定为 640 kbps、2,560 bytes；它与演示样本的 I0 速率和长度一致。AC-3 Table 4.13 的 48 kHz/640 kbps frmsizecod 为 36 或 37，两种编码长度均为 1,280 个 16-bit words。演示样本没有读取 frmsizecod，因此调用方必须显式提供合法的 36 或 37。

## 有界序列化范围

建议函数：write_flat7x_i0_ac3_header_through_compression。从 bit cursor 0 开始，按 ETSI MSB-first 语法写入：

syncword U16, crc1 U16, fscod=0 U2, frmsizecod U6, bsid=6 U5, bsmod U3, acmod=7 U3, cmixlev U2, surmixlev U2, lfeon=1 U1, dialnorm U5, compre U1, optional compr U8。

因此不带 compr 时结束于 bit 62；带 compr 时结束于 bit 70。整个 segment 在首个写入前做容量与取值校验。crc1、bsmod、cmixlev、surmixlev、dialnorm 与可选 compr 都是显式调用方输入；本包不给它们推导默认值，也不计算 CRC。

## 边界

helper 在 langcode 之前返回 cursor。它不生成 AC-3 audio blocks、auxdata、CRC1/CRC2 算法、完整 I0 frame、D0、AU、JOC/EMDF，也不接入生产编码器。现有 5.1 E-AC-3 writer 保持原路径。输入向量仅验证格式字段和 fail-before-write 行为，不包含媒体或音频载荷。production_allowed=false。
