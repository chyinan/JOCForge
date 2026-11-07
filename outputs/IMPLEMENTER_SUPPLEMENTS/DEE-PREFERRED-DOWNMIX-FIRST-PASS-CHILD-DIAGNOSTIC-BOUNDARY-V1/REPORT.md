# Preferred downmix selected-pass child 边界

在所检查的 selected encode-pass 调用点，FUN_1801475f0 将 Priv+0x350 的 record/subobject 指针交给 FUN_180153090，另一个参数指向 caller-local record。该子函数有多个 direct caller；本收据只覆盖当前 selected call edge。

该子函数的返回对象 +0x20 被 caller 用于生成 source-count/mismatch 诊断文本。selected caller 与该 child 在所检查函数体内均没有读取 MixMode 对应的 Priv+0x980，也没有把 downmix scalar 作为该 child 参数传递。该边没有建立 DSP、renderer 或 BSI 消费者。

精确边界：Priv+0x350 的 record 仍是不透明对象；该 selected edge 不传递 MixMode 值。音频/BSI consumer 与 downmix 效果保持 UNBOUND，不作产品范围内“没有其他消费者”的结论。Ghidra 12.2 只读/no-analysis；未运行 DEE/XML，未读媒体、PCM、载荷、私钥或许可内容。
