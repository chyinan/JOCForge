# 官方演示样本：D0 block-2 skipfld 两 AU 数值向量

## 数值观察

目标的 `block_index=2` 已按零起始合同确认，即每个 syncframe 的第三个 audio-block prefix。官方样本 AU0 与 AU1 的 D0 都在该位置观察到 `skipfld=true`，且 EMDF 容器完整消费该 skip field、trailing bytes 为 0：

| AU | D0 file offset | sample frame size/rate | C / skiple | ID 11 data | ID 14 data | consumed | trailing |
|---:|---:|---|---:|---:|---:|---:|---:|
| 0 | 7984 | 2560 bytes / 640 kbps | 314 B | 79 B | 216 B | 314 B | 0 B |
| 1 | 13104 | 2560 bytes / 640 kbps | 301 B | 66 B | 216 B | 301 B | 0 B |

## 使用范围

这是同一官方样本两个 AU 的结构性数值向量。它可用于 skip-field container/EMDF parser 的 sample regression。它没有 raw skip-field bytes、packet bytes 或 PCM。AU0 与 AU1 的 C 不同，二者保持为独立样本值。

`C ≤ 511` 且超限 fail-closed 是 Flat7x 目标合同；这些 640-kbps 样本均落在该范围内，并不证明 1024-kbps D0 的 C 分布或 native capacity。禁止把跨 skipfld 分片作为目标实现。D0 音频span、ADM角色映射和生成算法仍由独立实现/证据提供。
