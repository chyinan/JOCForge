# F07 单通道瞬态块处理

本切片把 F07A 的 D05 provider 和 F07.2/F07.3 单通道块处理分开。provider 先验证完整文件长度与 SHA-256，再检查 6 组参数的字段顺序、位模式和规范序列化摘要。它只暴露当前选定的 33 个 binary32 参数，不读取文件或环境。

块处理器接收一条 FBW lane 的 256 个 F32 位模式、上一块状态和已验证参数。它执行两节逐样本滤波、三节成对滤波、history clear、四路 peak、tier 判断和两个独立四槽队列。它返回 filtered 位模式、标志、位置、均值和 next state。调用方逐块传入 next state。

定向测试从冷状态连续处理 36 个 input-only 块。它比较每块的 filtered hash、124-byte 状态 hash、mean、flags、tiers 和 peak positions。实际 D05 provider 的 36-block 结果全部匹配。

R3 actual review 记录的 F07 规格 SHA-256 是 `ab59cb962673315bfa9b8be3018aaffa778e8d29bb92903de0c4165224dd1530`。当前规格 SHA-256 是 `61a957410a05e1e44c15d7a3f6622d2ff4c2a1135020f35261c0dee1979f1eac`。当前测试说明本地代码匹配已交付 36-block 结果，不宣称 R3 review 覆盖当前规格。

这个 slice 不处理另外四个 FBW lane、F07.1 输入历史、F07.4 结构标志、programme state、reset/EOF、F08 变换或 parent transaction。父 F07 和音频链路仍未闭合。
