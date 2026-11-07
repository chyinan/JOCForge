# 七份授权 ADM BWF 的 metadata-only 轨数清单

该收据只覆盖用户授权的七份 D: ADM BWF。研究侧读取 RIFF/WAVE 头及 fmt、bext、CHNA、AXML、ds64 元数据；对每个 data chunk 只按元数据声明的长度 seek 跳过，PCM 实际读取为 0 字节。没有全文件哈希、复制、改名或 DEE 运行。

七份样本的 WAVE fmt 物理声道数只有两个取值：N=36（6 份）和 N=38（1 份）。每份的 CHNA track/UID 计数及 AXML audioTrackUID/audioChannelFormat 计数均与该物理 N 相等；全部为 48 kHz、24-bit PCM。

最短样本是唯一的 38 轨“人是猫”，6,522,906 帧/声道，2:15.894；最短 36 轨样本是“My jealousy”，7,328,771 帧/声道，2:32.683。这个列表只证明这七份输入的 metadata 一致性，不代表 DEE 或 JOCForge 对这些 N 的运行时接纳，也不代表任意轨数支持范围。
