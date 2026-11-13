---
status: provisional
topic: "Flat7x D0 four-channel audio policy"
spec: "ETSI TS 102 366 V1.4.1; ETSI TS 103 420 V1.2.1"
confidence: "E-AC-3 syntax verified; source projection and encoding choices provisional"
tags: [JOCForge, Flat7x, D0, E-AC-3, ETSI]
---

# Flat7x D0 四声道音频策略

## 已封存的实现契约

Wash 为实现端封存了 D0 四声道编码契约：[CONTRACT.md](../../outputs/IMPLEMENTER_SUPPLEMENTS/FLAT7X-D0-FOUR-CHANNEL-ENCODER-POLICY-V1/CONTRACT.md)。Manifest SHA-256 为 689d5e4f3b27fd79110f6fb99189d4407384424929b3a74d20ba3447698f545b，package seal SHA-256 为 5f0c525745784e7b41714d04e5dc432fc2689ccf6ae74e925afe3cf7662e82b6。普通模式和 python -O checker 均通过。

该契约让实现 Agent 继续 D0 producer。D0 coded channels 为 4 个，顺序是 Ls、Rs、Lrs、Rrs；JOC reconstruction carriers 仍为 7 个。ADM source-role map 暂按可替换假设使用，并通过 typed receipt 传递。

D0 音频策略采用 ETSI 允许的 6×256 样本 block 结构、512 点长变换、D15 指数和 ETSI bit-allocation/mantissa 语法。编码器状态按 source generation 独立持有四路 overlap/history；I0、D0、F13 必须绑定同一 AU 后原子提交。EOF 尾部补零并记录 valid/padded sample 数。

D0 目标使用 chbwcod=60，对应 253 个 active bins。隔离仓库已有 F12A BAP provider 限定为 217 个 full-band bins 加 LFE7，且其分发范围标记为 NOT_AUTHORIZED。因此新代码需新增 D0 专用 4×253、无 LFE provider；5.1 provider 保持原样。

## 当前进度

编码算法代码、输入向量执行、DEE/encoder run 和 .ec3 输出均为 NOT_RUN。真实编码仍需单独 Fresh GO。实现 Agent 已收到该契约，并开始按公开 ETSI 表构建 D0 专用 BAP 路径。

2026-11-14 进度：帧尾策略已物化为 `Flat7xD0FrameTailPolicy::ZeroFillUnusedAuxBits`，从 source-bound D0 receipt 传到 AU/frame composer；生产 writer 写入 `auxdatae=0` 并显式零填剩余 auxbits，CRC2 保持最后 16 位。block 2 唯一 skip-field 和 1–511 字节范围继续由现有 assembler 检查。原始 opaque tail 注入只在 `test-support` 下用于结构复制用例。D0 专用 4×253-bin BAP table provider 和 6×4 typed BAP row allocator 已实现，ETSI table-set SHA-256 为 `5a9ea2d179dc8017316006e8ffd23b2f5e30f1b363e56740f94787b4ef8e96fe`。PCM transform、D15 exponent production、mantissa/audblk serialization、六块历史、F12/F13/F15 同源提交和真实输出仍未闭合。
