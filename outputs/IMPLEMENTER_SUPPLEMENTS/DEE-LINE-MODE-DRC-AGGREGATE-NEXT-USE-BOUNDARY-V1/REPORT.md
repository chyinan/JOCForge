# Line-mode aggregate 后续指针传递边界

沿已确认的诊断首读之后，在同一调用函数内，下一次把同一 aggregate 地址传递给另一个函数时，caller 将它放在第二实参寄存器 RDX；第一个实参 RCX 指向不同的目的栈对象。

被检查的 callee 只有一个正式参数。其函数体只初始化 RCX 目的对象的字段，并调用一个嵌套 helper；没有读取 RDX，因此这条调用不建立 aggregate 复制、类型转换或数据消费关系。嵌套 helper 未检查，后续 aggregate 用途也未检查。

该边界没有连接到 per-frame DRC +0xA0、codec 或 BSI。聚合类型、目的对象类型、owner/generation 及下一 helper 的作用仍未闭合；不据此声称全局没有消费者。来源为 Ghidra 12.2 只读静态收据。未运行 DEE 或控制作业进程，未读取媒体、PCM、载荷、私钥或许可内容。实现处置：只记录该调用边，不改 DRC/codec 算法。
