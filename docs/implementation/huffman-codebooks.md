# F05QH：JOC Huffman表与位序

本条款提供F05Q实际消费的6组Huffman表。参数文件为 `parameters/joc-huffman-v1.json`，表级输入向量为 `test-vectors/JOC-HUFFMAN-INPUT-VECTORS.json`，consumer连接及事务向量为 `test-vectors/JOC-HUFFMAN-CONSUMER-CONNECTION-VERIFICATION.json`。这些表是普通协议功能常数，不包含认证秘密。

## F05QH.1 来源和范围

6 组表对应 ETSI TS 103 420 V1.2.1 第 6.6.3 节和 Annex A.1。官方伴随 ZIP 的固定 SHA-256 为 `a79cf108c4529b7d9ca9525c871183a70b1732ed6df03a3d85b2f31be46eeced`。其中 `ts_103420_tables.c` 的固定 SHA-256 为 `4db8ae83e3c2e9269e88365be92a1a3ed6a9e6ee3851afac8ca03902723b1fcd`。

OpenJOC 默认参考路径未保留官方 ZIP，因此本轮没有重新执行解包。两份公开生成表逐字节相同。每个公开树都从根节点 0 完整遍历。研究模型的 588 个 codeword 只用于逐项 post-compare，不用于生成公开树。

## F05QH.2 表映射

| F05Q 名称 | Annex A.1 名称 | 符号数 | 树节点数 | 消费位置 |
|---|---|---:|---:|---|
| `full_coarse` | `joc_huff_code_coarse_generic` | 96 | 95 | mode 0 系数差分 |
| `full_fine` | `joc_huff_code_fine_generic` | 192 | 191 | mode 1 系数差分 |
| `sparse_coarse` | `joc_huff_code_coarse_coeff_sparse` | 96 | 95 | mode 2 系数差分 |
| `sparse_fine` | `joc_huff_code_fine_coeff_sparse` | 192 | 191 | mode 3 系数差分 |
| `position_5` | `joc_huff_code_5ch_pos_index_sparse` | 5 | 4 | 5 通道 sparse 位置差分 |
| `position_7` | `joc_huff_code_7ch_pos_index_sparse` | 7 | 6 | 7 通道 sparse 位置差分 |

## F05QH.3 参数表示

每个符号记录 `symbol`、`code`、`bit_length` 和 `msb_first_bits`。`symbol` 从 0 连续递增。`code` 是非负整数。只取其低 `bit_length` 位，并以高位在前的顺序写出。`msb_first_bits` 是同一值补足到 `bit_length` 后的文本身份，必须与整数表示一致。

实现必须验证以下条件：表名集合精确等于 6 个固定名称；符号连续且数量匹配；`1 <= bit_length <= 24`；`0 <= code < 2^bit_length`；codeword 唯一且 prefix-free。6 组表都是完整二叉树，Kraft 和等于 1。只按长度重新生成 canonical Huffman code 会改变表值，因此禁止这么做。

## F05QH.4 写入和读取

写入一个符号时，先检查 `0 <= symbol < symbol_count`。随后从 `bit_length - 1` 到 0 读取 `code` 的位，逐位追加到当前 bitstream。codeword 之间不对齐。整个 ID14 body 完成后，最后一个 byte 的剩余低位补 0。有效 bit 数不含这些补位。

读取从当前根开始。每读 1 bit，就选择对应分支。到达 leaf 后返回 `symbol`，下一个符号重新从根开始。输入在内部节点结束时报 `HUFFMAN_TRUNCATED_CODEWORD`。节点引用越界、循环、重复 leaf、缺失符号或 prefix 冲突时报 `HUFFMAN_TABLE_INVALID`。符号小于0、等于或大于 `symbol_count`，以及布尔值等非整数输入时报 `HUFFMAN_SYMBOL_OUT_OF_RANGE`。

writer先复制当前bytes、valid-bit cursor和consumer state到局部trial，再执行表选择、symbol验证、写入和容量检查。成功时一次替换三者。任何错误都返回 `output_present=false` 和 `commit_count=0`，原writer bytes/cursor及consumer state逐字节不变。decoder同样使用局部cursor和局部target；只有完整到达一个leaf才提交两者。截断或坏表不消费输入，也不保留部分target。

系数scratch超限报 `JOC_COEFFICIENT_SCRATCH_EXCEEDED`，最终byte容量超限报 `JOC_PAYLOAD_CAPACITY_EXCEEDED`。这两个错误交给F05Q.7 retry；retry用尽后的外层错误为 `JOC_MINIMUM_REPRESENTATION_EXCEEDS_CAPACITY`。provider安装时先完整验证六表；`HUFFMAN_TABLE_INVALID` 的优先级高于任何symbol或payload处理。

## F05QH.5 F05Q 消费连接

full mode 每个时间点、每个 channel 把 `previous` 重置为 `S/2`。`S` 为 96 或 192。每个 band 写 `(q - previous) mod S`，随后令 `previous = q`。mode 0 选 `full_coarse`，mode 1 选 `full_fine`。

sparse mode 的首个 channel 索引仍直接写 3 bit。后续 band 写 `(current_channel - previous_channel) mod channel_count`。5 通道使用 `position_5`，7 通道使用 `position_7`。系数差分从 50（mode 2）或 100（mode 3）开始。mode 2 选 `sparse_coarse`，mode 3 选 `sparse_fine`。

Huffman bit 数只包括 codeword。容量判定沿用 F05Q：`coefficient_bits <= 4344`，且 `ceil(total_valid_bits / 8) <= capacity_bytes`。padding 不能算入有效 bit 数，但会占最终 byte 容量。

## F05QH.6 验收边界

表级验收逐表覆盖全部588个符号，验证encode/decode往返、prefix-free、完整树和逐表稳定SHA-256。18个连续流案例覆盖升序、降序和边界重复。12个容量案例覆盖刚好装入、少1 byte和4,344 bit scratch超限。18个非法符号和6个截断codeword必须按上节拒绝。

consumer连接向量另覆盖mode 0/1/2/3到四组系数表、5/7通道到两组position表、full每point/channel reset、sparse的50/100初值。失败向量分别覆盖非法symbol、最终容量、scratch容量、截断和坏表；每项保存规范错误码及失败前后的writer、cursor和state快照。

这组材料关闭 6 个 Huffman 表的数据身份、位序和局部容量计算。它不关闭 fit producer、量化、profile 降级、QMF、JOC 空间验收或整个编码器的 IMPLEMENTATION_READY 状态。

## F05QH.7 局部独立复审状态

r2 独立复审登记见 `test-vectors/JOC-HUFFMAN-LOCAL-REVIEW-STATUS.json`。该记录绑定冻结 manifest、独立 checker/readset/result 身份、102 个 write events、28 个 reset events、5 个零提交失败案例和 9 个 checker mutation；局部 Huffman/F05Q consumer 为 `YES`，whole implementation 保持 `NO`。本节只登记审查状态，不改变本条款的表、位序、遍历、容量或错误算法。
