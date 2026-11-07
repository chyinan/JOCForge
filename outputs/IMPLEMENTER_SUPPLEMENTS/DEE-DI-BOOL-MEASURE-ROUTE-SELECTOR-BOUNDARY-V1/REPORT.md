# Dialogue Intelligence Bool 与测量/编码路由

复用的 Ghidra 12.2 静态链把 FilterDDPJocEnc+0x8 的模式值写入 Priv+0xf34。该被检查的路由器在模式为 0 时进入 Priv::measure，在模式为 1 时进入 Priv::encode。

dialogue_intelligence Bool 是独立字段 Priv+0x2e0，并被放入 measurement setup+0x94。该路由边只读取 +0xf34，没有读取或按 Bool 分支；现有 reader-factory 收据中的 reader variant 字段为 setup+0x9c 和 +0x140，也没有在这条边上绑定 Bool 到 reader variant 的分支。

结论限于这一 owner/路由边：Bool 是测量输入，不是这里的 measure-versus-encode 选择器。Bool 到最终 measurement/audio result 的因果效果仍为 UNBOUND；不声称其他函数从不读取该 Bool。未运行 DEE 或 runtime capture，也未读取媒体、PCM、载荷、私钥或许可内容。实现处置：保持 Bool 与 route mode 分离，不据此更改算法。
