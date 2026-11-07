# Host start/end 属性到 Reader range 边界

StageRunner 的静态 path 经过 descriptor factory vtable +0x30，把接口保存在 StageRunner+0x270；RunAudioFilter 随后通过 generic SetProperty vtable +0x8 调用属性对象。该受限 path 只记录 source 与 target 的静态 key，没有观察到 start/end key 或值。

plugin range 收据定位了 Priv+0xf98/+0xfb0 的 sample-bound producer；Reader 收据也定位了 range record 到 Reader input 构造的静态路径。但 generic host property 值没有 typed 连接到这两个成员，也没有连接到 Reader cursor +0x28 / total +0x20。selected start/end、owner/generation 和实际 sample position 仍 UNBOUND/NOT_RUN。

这只覆盖所检查的 StageRunner property dispatch path，不作全局无 key 结论。没有重跑 time_base hook、sample-range 算法、DEE 或 Ghidra；没有读取媒体、PCM、payload、key 或许可内容。
