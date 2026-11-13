# 官方演示样本 I0 AC-3 BSI continuation

## 观测结论

Wash 将三份独立字段账本按精确 bit cursor 合并到 I0 `addbsie`。样本 I0 为 AC-3 BSID6、`acmod=7`、LFE on、48 kHz；`xbsi1e=1`、`dmixmod=1`，四个 Annex-D mix level 均为 code 4（0.707 / -3 dB）。`xbsi2e=1` 后的 14 bits 已逐字段读取；紧随其后的 `addbsie=0`，因此样本没有 addbsi payload，I0 BSI 到 audblk 边界闭合。

输入仅由 SHA-256 标识。包内保存数字字段、位宽和 bit offsets，不含源路径、MKA、帧字节或解析脚本。

## Flat7x 目标应用

用户目标为 I0 640 kbps + D0 1024 kbps = 1664 kbps，并明确要求将演示样本的非码率 I0 参数用于该目标。此补充提供该 I0 BSI 字段组的样本值；样本本身为 1280 kbps，1664 kbps 输出仍未观测。

本补充不定义 audblk 音频生成算法，也不提供 D0 音频规则。输出帧与 AU 验证仍是独立后续项。

ETSI TS 102 366 V1.4.1 Annex D: https://www.etsi.org/deliver/etsi_ts/102300_102399/102366/01.04.01_60/ts_102366v010401p.pdf
