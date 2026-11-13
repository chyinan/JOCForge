# 官方演示样本 I0 Annex-D BSI 字段

## 观测值

Wash 从用户指定的官方 MKA 中识别出 I0 AC-3 BSID6 帧，并在 `xbsi2e` 后停止。源文件以 SHA-256 标识；包内只包含字段值、位宽和位偏移。

I0 头部为 48 kHz、`frmsizecod=36`、`acmod=7`、LFE on。Annex-D `xbsi1e=1`，`dmixmod=1`（Lt/Rt preferred）；`ltrtcmixlev`、`ltrtsurmixlev`、`lorocmixlev`、`lorosurmixlev` 均为 code 4，即 0.707（-3 dB）。`xbsi2e=1`，它后续的 14 bits 尚未捕获。

ETSI TS 102 366 V1.4.1 Annex D 定义该字段语法及值解释：[官方 ETSI 规范](https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf)。

## 目标使用范围

用户指定的 1664 kbps 目标为 I0 640 kbps + D0 1024 kbps；用户指定将官方演示样本的非码率 I0 参数用于该目标。此包因此为目标 I0 提供 hash-bound 的 Annex-D 字段值。样本本身为 1280 kbps；它不构成 1664 kbps 输出观察。

本包只授权这些 I0 BSI 值。`xbsi2e` 后的字段、完整 I0 帧、D0 音频生成及完整 Flat7x 输出保持独立状态。
