# F09 raw exponent from POWER bits

本切片实现一个纯整数 helper。它接收调用方提供的 U32 POWER 位模式、I16 bias 和 I16 cap，返回 I16 raw exponent。函数不把输入转换成浮点数，也不计算 POWER。

实现顺序如下：将 U32 位模式乘以 2 并按 32 位回绕；从结果高字节提取指数，零字映射为 `-127`；计算 `clamp(-source_exponent, 0, 60)`；将 bias 与差值按 I16 回绕相加；用有符号比较应用 cap；最后做 I16 算术右移 1 位。

验收覆盖 5 个 recipes，共 50 组输入/输出。输入包括正零、负零、次正规数边界和若干选定指数位模式。预期输出只用于计算后比较。

调用方仍负责生成 POWER 位模式、选择 bias、cap 和 route。包外的输入错误、POWER writer、bias provider、LFE/系数对分支及完整 F09 producer 都不在本切片内。Whole `IMPLEMENTATION_READY` 仍为 `NO`。
