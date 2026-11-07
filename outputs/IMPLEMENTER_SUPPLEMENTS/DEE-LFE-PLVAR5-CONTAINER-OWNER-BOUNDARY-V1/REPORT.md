# LFE plVar5 容器所有者边界

Ghidra 12.2 的只读收据显示，FUN_140350d00 入口没有 CALL 引用；三条 DATA 引用均落在展开信息元数据中，不能作为调用方、对象所有者或构造关系。

在该函数局部，plVar5 由 opaque param_1 + 0x10a6 得到。当前材料没有把 param_1 连接到有类型的构造器、容器填充者或 generation。最早未闭合边是 FUN_140350d00.param_1；plVar5 的 owner 与 generation 保持 UNBOUND。

这是所检查的一条边界，不是全映像不存在 writer/caller 的结论。未做运行时观测或 DEE 作业；未读取媒体、PCM、载荷、私钥或许可内容。实现处置：只记录边界，不修改 LFE 行为，也不把相同偏移推断为其他对象的身份。
