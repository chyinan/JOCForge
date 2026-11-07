# ADM 36 轨短窗动态 frame-shape 观察

本次 DEE attempt-05 使用 `atmos_mezz` 输入和配置为 `[0, 50000)` 的 `file_position` 短窗，普通 measurement 与 encoder pass 完成，exit 0，attempt-local 输出为 135,893 bytes。观察器未读取或保存 PCM、ADM essence、压缩音频 payload、私钥或许可内容；输出没有解码或试听。

本次观察到 WAVE fmt、CHNA physical rows、U helper、K descriptor、composite B 与非空 frame-shape actual count 均为 36。Tgen-1/Jgen-1 中，K→B→AtmosProcessor D 有 owner-token join，33 个非空 D 调用均为 36 channels / 1,536 samples。首个非空调用的 processor expected count 为 0；同一 processor/frame descriptor 的下一调用及余下 31 次为 36。十次空调用为 0 channels / 0 samples 且无 B descriptor。

U→K owner join 仍未绑定：四条 K writer 记录的 U owner token 都为空，数值相同不能代替对象身份。之后出现 Tgen-2/Jgen-2：B descriptor token 被复用，但 row-table token 不同，本短窗没有后续 D consumer。time_base hook 的 4 次 key read 均为 `READ_FAILED`，原生 time_base 和绝对 ADM sample position 未捕获。

这只证明单个 N=36 短窗的动态形状路径；不证明其他 N、通用轨数范围、模型/DEE 音频一致性或完整输出链。PCM 比较、Type10 composition 与六路数值对照均未运行。
