# Dialogue Intelligence Bool 到 measurement result producer 边界

该增补追踪 per-record `Float32` 结果字段 `lVar7+0x14` 的两个静态 producer 候选。两条路径都能解释结果字段写入，但都没有把 setup Bool `+0x94` 连接到该标量。

EmpireLoudness 路径从 private state `+0x88` 写 per-record `+0x14`；Bool `+0x94` 被复制到同一 owner 的 `+0x2c`，但 `+0x2c → +0x88` 的 typed dependency 未建立。AtmosProcMeasure 路径中，Bool `+0x94` 影响配置首字节；所检查的构造函数读的是配置 `+0x14` 常量 1。结果字段来自 meter context `+0x40`，经 result structure 写入 per-record `+0x14`。零计数 flush 也走同一 finalizer。

因此，记录生产者静态有界，Bool 到 result scalar 的因果边仍 UNBOUND。runtime 选择哪个 producer、Bool 的最终测量/音频效果及全局是否存在其他消费者均未建立。未运行 DEE/UI/runtime 或读取音频、PCM、载荷、私钥/许可。
