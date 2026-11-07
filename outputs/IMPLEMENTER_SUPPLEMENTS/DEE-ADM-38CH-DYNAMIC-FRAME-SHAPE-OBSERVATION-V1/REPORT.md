# ADM 38 轨短窗动态 frame-shape 观察

本次 DEE attempt-04 使用 `atmos_mezz` 输入和 file_position 短窗，完成普通 measurement 与 encoder pass，exit 0，生成 135,893-byte attempt-local 输出。Wash observer 没有读取或保存 PCM、ADM essence、EC-3/MP4 payload、私钥或许可内容；输出文件本身未解码或试听。

本次观察到 fmt、CHNA、U helper、K descriptor 与 composite B 的计数值均为 38。Tgen-1/Jgen-1 中，单个 K writer 事件经 owner token 与 B count 38、processor/frame descriptor 的 33 次非空调用连接。U→K 仍没有同一 owner token：四条 K writer 记录的 U owner token 均为空，数值相同不能替代 owner/generation join。

33 次非空 frame-shape consumer 调用中，首个 actual channels=38、processor expected=0；同 processor/descriptor 的下一次 expected=38，后续 31 次也为 38。另有 10 次空调用为 0 channel / 0 sample，且没有 B descriptor。Tgen-2 在这些 shape 调用后构造，复用了 B descriptor token，但 row-table token 不同；本短窗未观察其 D consumer。

这是一份 N=38、50,000-sample 短窗的动态事实，不证明 N=36 或其他 N 的 runtime 接纳。1..128 gate 的全路径意义、U→K owner join、source_record_count、active-target count、实际 layout flag、绝对 ADM sample position、time_base 原生值和完整音频效果仍未闭合。
