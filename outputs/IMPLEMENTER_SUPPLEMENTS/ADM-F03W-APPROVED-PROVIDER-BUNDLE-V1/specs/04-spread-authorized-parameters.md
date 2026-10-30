# F04GA：F04G 四组参数已供应

本次用户明确授权，除私有key或认证秘密外的正常非秘密常数可向内部隔离实现交付和使用。因此四组共24个F32、96 bytes的功能值已整理为 `parameters/spread-authorized-v1.json`。分类为获准的非秘密音频功能参数，不宣称为独立数学生成。D10-W/WK绑定已有静态音频证据，D10-PC/SC绑定已核验冻结Python模型；本轮未重新确认原编译位置。供应只限内部，不授权公开或第三方分发，不包含原表文件、路径、地址、源码、DLL或认证秘密。

所有位模式与顺序见参数文件。F04G.1～9的运算顺序继续完整适用；前一份连接补充草稿中“原值未授权”的历史状态由本条覆盖，接口、域、完整函数和失败语义保持。几何由独立provider供应，不能以本次原创合成11点代替生产layout。

`test-vectors/SPREAD-AUTHORIZED-VERIFICATION.json`包含三组完整input-only recipes、输出U32LE SHA256，以及9组完整11点gain案例。计算只消费参数、明确的原创合成几何和输入；expected数值/hash仅用于计算后的比较。完整案例从三轴response、各cell、boundary/interior归一化、spread和column、sin²、unadjusted到final gain逐项核验。案例0与8重复相同输入，中间穿插其他输入，验证纯producer的无数值历史状态；不把它当作gain ramp或PCM管线连续验收。三个正raw-width rejection覆盖量化q0、q1、q2的mapped-width threshold拒绝。

宽度32768项、固定power65534项、mapped-width p及reciprocal 65530项逐位匹配冻结模型；9完整gain案例含81组字段比较、1764个F32词，零差异。完整gain是独立规格代数重算对冻结模型比较，参数来自获准供应；不是以模型输出作为预测种子。

研究侧附有纯参数化数学core、无I/O full-gain core及复算shell，供根Agent复验；这些研究验证程序不属于产品实现，也不交付原模型源码。一般数学pow在之前固定power比较的65534项中65527項位模式不同，该不等价结论保持。

本次仅闭合功能值供应及声明合成域的参数消费者。没有新真实ADM/PCM重跑或完整空间质量验收，其他上游几何和管线状态仍须各自闭合；整包 `IMPLEMENTATION_READY` 不能据此设置YES。
