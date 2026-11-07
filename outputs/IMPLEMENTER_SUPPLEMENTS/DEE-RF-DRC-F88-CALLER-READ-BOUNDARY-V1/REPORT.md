# RF DRC caller-local +0xf88 读取边界

在所检查的 caller FUN_180135d40 中，RDI+0xf88 被赋给局部别名 RBX；该处先把两个 qword 清零，将地址作为 RCX 传给 FUN_18013ee70，返回后再把 RAX 写回 [RBX]。callee 保存 RCX，但所检查函数体不回读该值；它分配并初始化一个独立对象。

本 caller 的局部扫描未找到通过 RBX 对该字段的读取；栈地址保存槽在后续被覆盖，另一处 LEA 只是取地址。该 caller 内该字段的语义类型，以及它与 RF DRC destination vector、StringType 或 numeric state 的连接仍 UNBOUND。此结果只覆盖该 caller，不作全局无消费者结论，也不按邻近偏移推断对象身份。

Ghidra 12.2 只读/no-analysis；DEE/runtime、XML、decode、PCM、载荷、私钥/许可均未运行或读取。实现处置：保持 owner/type 与 numeric RF effect 未绑定，不改算法。
