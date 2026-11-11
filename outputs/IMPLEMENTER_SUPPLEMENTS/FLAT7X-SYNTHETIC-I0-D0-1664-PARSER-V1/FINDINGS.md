# Flat7x synthetic 1664-kbps I0+D0 parser boundary

## 洗白结论

OpenJOC 0.20.1 对一个纯内存构造的 6,656-byte target-size AU 完成了受限结构检查：I0 LegacyIndependent/BSID6/2,560 bytes 后接 D0 Dependent/BSID16/4,096 bytes；两帧被分组为一个 48-kHz、1,536-sample、32-ms AU。D0 BSI 中的 chanmap 为 0x1A00。D0 block 2 单个 skipfld 的 C_total=511 bytes 由 EMDF/JOC exact-carrier parser 精确消费；ID14 报告 index 1、7 channels、15 present objects。

这是 synthetic parser compatibility evidence。它支持把目标帧大小、I0→D0 排列和 C=511 解析结果用于结构回归；它没有生成真实音频文件，也没有验证完整帧音频块或 I0 CRC。

## 类型化事实与边界

- Target-size vector：I0 2,560 bytes + D0 4,096 bytes = 6,656 bytes/AU，48 kHz 下 1,536 samples / 32 ms，aggregate rate 1,664,000 bit/s。
- I0：LegacyIndependent，BSID 6，frmsizecod 36，acmod 7，lfeon=true；BSI parse PASS；CRC validation NOT_RUN。
- D0：Dependent，substreamid 0，BSID 16，6 blocks，chanmap 0x1A00；BSI parse PASS。
- 单一 D0 block-2 skipfld：skipl=511 bytes；ID11+ID14 synthetic body；exact-carrier result Parsed，511/511 bytes consumed。ID14 parser result 为 index 1、7 channels、15 present object records。
- ID11 的 466 bytes 是 opaque zero-fill；OAMD semantics NOT_RUN。complexity 16、semantic owner/source generation 均 UNBOUND。
- Full-frame audio-block parser、I0 CRC validation、decoder、PCM、real media 和 real product output 均 NOT_RUN。
- 用户指定的官方 MKA 参考是独立 1,280,000-bit/s 观察（I0 640 + D0 640）；该参考未用于构造本 vector。

## 使用范围

可将这些字段作为 Flat7x structural parser/assembler 的输入期望值。该包不提供编码算法、真实帧字节或输出授权；production_allowed=false。C=511 positive parse 不替代 C=512 fail-closed 边界验证。
