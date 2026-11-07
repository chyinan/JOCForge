# Composition layout branch flag 写入路径

所检查的 composite-master 对象在同一条静态 factory 路径上分配并构造。构造器对 +0x100 做 16-bit 零初始化，覆盖 branch byte +0x100 与相邻字节 +0x101。通过 admission gate 后，首个同对象写入点计算并写入 +0x100 的 byte。

写入规则不是复制一个单独布尔值：outer factory byte 参数的 bit 0x2 可生成 option-map key 2；同时，source shape count 与六字节 mapping entry 的比较可产生 mismatch。writer 在两者任一为非零时写 1，否则写 0。另一个参数 3 用于对象 +0x8，不是 +0x100 的来源。

该规则闭合到静态 owner/构造/首写路径，但实际任务是否选中该 branch value 仍 NOT_CAPTURED；不据此认定 36 或 38 轨运行走了哪条分支。未运行 DEE 或读取 PCM/载荷/私钥/许可内容。实现处置：只作为静态条件路径证据，保留运行时 branch 与最终音频效果未验证的边界。
