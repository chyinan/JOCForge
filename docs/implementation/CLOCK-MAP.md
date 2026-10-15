# 时钟映射

时间域关系以 ../architecture/DATAFLOW.md 为准。公共 API 的时间索引必须使用不同 newtype。

| Functional ID | Clock domain | Typed boundary |
|---|---|---|
| STRC | programme/attempt | `programme_attempt_receipt` |
| GTC | programme/attempt | `programme_attempt_receipt` |
| XSTJ | programme/attempt | `programme_attempt_receipt` |
| F01 | callback g | `callback_g_receipt` |
| F01R | callback g | `callback_g_receipt` |
| F01G | callback g | `callback_g_receipt` |
| F01B | callback g | `callback_g_receipt` |
| F01GL | callback g | `callback_g_receipt` |
| F01C | callback g | `callback_g_receipt` |
| F01E | callback g | `callback_g_receipt` |
| F02 | callback g | `callback_g_receipt` |
| F03 | callback g | `callback_g_receipt` |
| F03G | callback g | `callback_g_receipt` |
| F03C | callback g | `callback_g_receipt` |
| F03U | callback g | `callback_g_receipt` |
| F03F | callback g | `callback_g_receipt` |
| F03FC | callback g | `callback_g_receipt` |
| F03R | callback g | `callback_g_receipt` |
| F03P | callback g | `callback_g_receipt` |
| F03N | callback g | `callback_g_receipt` |
| F03S | callback g | `callback_g_receipt` |
| F03FD | callback g | `callback_g_receipt` |
| F03W | callback g | `callback_g_receipt` |
| F04 | callback g | `callback_g_receipt` |
| F04C | callback g | `callback_g_receipt` |
| F04A | callback g | `callback_g_receipt` |
| F04F | callback g | `callback_g_receipt` |
| F04FM | callback g | `callback_g_receipt` |
| F04FCM | callback g | `callback_g_receipt` |
| F04G | callback g | `callback_g_receipt` |
| F04GA | callback g | `callback_g_receipt` |
| F04P | callback g | `callback_g_receipt` |
| F04PG | callback g | `callback_g_receipt` |
| F04R | callback g | `callback_g_receipt` |
| F04D | callback g | `callback_g_receipt` |
| F04Q | callback g | `callback_g_receipt` |
| F04RB | callback g | `callback_g_receipt` |
| F04QF | callback g | `callback_g_receipt` |
| F04QP | callback g | `callback_g_receipt` |
| F05 | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05Q | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05QH | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05J | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05JC | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05JL | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05JS | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05O | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05T | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05H | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05L | dispatch d / writer | `dispatch_d_writer_receipt` |
| F05R56 | callback g | `callback_g_receipt` |
| F06 | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06L | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06I | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06W | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06KG | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06TP | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06ND | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06NR | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06NI | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06NF | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06WH | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06U | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06DG | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06D | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06M | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OG | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OM | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OP | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OE | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06QT | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06QM | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06QS | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OD | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OS | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OB | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OR | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OC | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F06OF | meter callback or source s | `meter_callback_or_source_s_receipt` |
| F07 | source s / block | `source_s_block_receipt` |
| F07A | source s / block | `source_s_block_receipt` |
| F08 | source s / block | `source_s_block_receipt` |
| F08NZ | source s / block | `source_s_block_receipt` |
| F09 | source s / block | `source_s_block_receipt` |
| F09E | source s / block | `source_s_block_receipt` |
| F09PA | source s / block | `source_s_block_receipt` |
| F09G | source s / block | `source_s_block_receipt` |
| F09LB | source s / block | `source_s_block_receipt` |
| F10 | source s / block | `source_s_block_receipt` |
| F11 | source s / block | `source_s_block_receipt` |
| F12 | source s / block | `source_s_block_receipt` |
| F12S | source s / block | `source_s_block_receipt` |
| F12SC | source s / block | `source_s_block_receipt` |
| F12A | source s / block | `source_s_block_receipt` |
| F13 | source s / writer / AU j | `source_s_writer_AU_j_receipt` |
| F13F | source s / writer / AU j | `source_s_writer_AU_j_receipt` |
| F13T | source s / writer / AU j | `source_s_writer_AU_j_receipt` |
| F13C | source s / writer / AU j | `source_s_writer_AU_j_receipt` |
| F13D2 | source s / writer / AU j | `source_s_writer_AU_j_receipt` |
| F14 | source s / AU j | `source_s_AU_j_receipt` |
| F14K | source s / AU j | `source_s_AU_j_receipt` |
| F15 | source s / AU j | `source_s_AU_j_receipt` |
| F15P | source s / AU j | `source_s_AU_j_receipt` |
| OWNJ | callback g / source s | `callback_g_source_s_receipt` |

