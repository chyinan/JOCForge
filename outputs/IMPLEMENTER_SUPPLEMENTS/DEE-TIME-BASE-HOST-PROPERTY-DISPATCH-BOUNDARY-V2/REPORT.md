# time_base 宿主属性派发边界

这份 V2 修订包修正来源清单中的 plugin image digest 长度错误。canonical 64 位 image SHA 来自已封存的 plugin setter receipt；宿主 dispatch parent 自己报告的 plugin digest 长度不正确，作为 parent provenance caveat 保留，未修改 parent。

静态宿主路径：StageRunner::Build 构造 descriptor interface，经 vtable +0x30 factory 存入 StageRunner+0x270；RunAudioFilter 取该 interface 后调用通用 SetProperty vtable +0x8。所检查路径只观察到 dee_audio_filter_source 与 dee_audio_filter_target:<index> 静态 key，没有观察到 time_base key，也没有把 generic key/value 连接到 AtmosMezz Priv+0x550 StringType setter。

plugin setter 收据中的 constructor 默认 file_position 只是默认值，不代表选中 job XML 的输入。实际 selected XML time_base、factory concrete type、SetProperty target 与 owner/generation 仍 UNBOUND/DYNAMIC。该包只覆盖这条 host property dispatch 静态路径，不重开 sample-range 逻辑，不运行 DEE，不读取媒体/PCM/载荷/key/license。
