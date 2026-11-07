# LFE recurrence coefficient vectors

## 已核验的常量

来源包从已绑定的 `dee_ddp_encoder.exe` 静态映像中读取 5 段初始化、不可执行的 `.rdata` 数据，共 388 个 IEEE-754 binary32 数值：

- 零选择分支 `FUN_14036d9e0`：系数表 A、B 各 128 个 F32，另有 4 个共用标量。
- 非零选择分支 `FUN_14036d7c0`：系数表 A、B 各 64 个 F32。
- 每个元素都带有绝对地址、小端原始 32 位字和解码后的 F32 值。复核器逐项重算原始字与 F32 的对应关系。

本包只洗白静态系数值和来源身份，不含目标二进制、指令、反编译或导出脚本。

## 使用边界

上游静态分支包显示：两个受检 dispatcher 的 LFE lane 都把 recurrence selector 设为 0，走 `FUN_14036d9e0`；非零表分支在这两个 dispatcher 中属于独立 FBW lane 路径。该结论只覆盖这两个静态调用方。

实际 ADM 的布局/代次、运行时 selector、FMA/MXCSR 执行语义、PCM 数值一致性和约 0.819 残差的因果关系仍未闭合。数值 parity 为 `NOT_RUN`，根因仍 `NOT_ESTABLISHED`。这些常量可供后续实现和对照使用，但不构成音频 parity 通过证明。

未运行 DEE，未读取 PCM、编码载荷、私钥或 license，也未访问实现仓库。

