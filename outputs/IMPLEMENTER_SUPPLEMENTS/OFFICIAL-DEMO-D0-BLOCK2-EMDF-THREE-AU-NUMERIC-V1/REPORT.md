# 官方演示样本：D0 block-2 skipfld 三 AU 数值向量

官方样本 AU0、AU1、AU2 的 D0 frame 中，零起始 `block_index=2`（第三个 audio-block prefix）均有单字段 `skipfld`。AU0 的 `C=314`；AU1/AU2 的 `C=301`。三份容器均被完整消费，trailing bytes=0，EMDF payload ID 为 11 和 14。

| AU | D0 offset | sample D0 | C | ID11 | ID14 | consumed | trailing |
|---:|---:|---|---:|---:|---:|---:|---:|
| 0 | 7984 | 2560 B / 640 kbps | 314 B | 79 B | 216 B | 314 B | 0 B |
| 1 | 13104 | 2560 B / 640 kbps | 301 B | 66 B | 216 B | 301 B | 0 B |
| 2 | 18224 | 2560 B / 640 kbps | 301 B | 66 B | 216 B | 301 B | 0 B |

这是一个官方样本的三 AU 数值向量，适用于结构解析/测试。`C≤511`、超限 fail-closed 且不跨字段续写属于目标合同；这些 640-kbps 数值不预测 1024-kbps 的 C。数据包不含 raw frame/skipfld/payload bytes、parser 或 PCM。
