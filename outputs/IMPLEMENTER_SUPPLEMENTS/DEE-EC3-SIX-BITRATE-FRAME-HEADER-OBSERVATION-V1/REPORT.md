# Six-rate DEE E-AC-3 frame-header observations

Six DEE rate choices were observed on the same authorized My Jealousy 36-channel ADM, 48 kHz, 5.1 Atmos DDP path, with a 33-frame short window and an M4A container. The evidence comes from the successful 1024 kbps attempt-05 baseline, the 768 kbps single-variable run, and a four-arm serial CLI sweep at 384/448/576/640 kbps. The per-run Wash manifests bind each XML and source receipt.

| DEE data_rate | E-AC-3 syncframes | frame size code | bytes/frame |
|---:|---:|---:|---:|
| 384 kbps | 33 | 767 | 1,536 |
| 448 kbps | 33 | 895 | 1,792 |
| 576 kbps | 33 | 1,151 | 2,304 |
| 640 kbps | 33 | 1,279 | 2,560 |
| 768 kbps | 33 | 1,535 | 3,072 |
| 1024 kbps | 33 | 2,047 | 4,096 |

Wash independently streamed retained M4A tracks through copy-only demux and checked every E-AC-3 syncframe size from its frame header, discarding frame bodies. All frame-size histograms matched the corresponding packet-size receipts; no packet bytes or PCM were persisted. The observed relationship on this 48 kHz/31.25 fps carrier window is 4 bytes per frame per kbps.

This demonstrates successful outputs and consistent carrier frame sizes for all six declared rates on this one input/profile/window. The registry callback target and typed `data_rate`-to-carrier-writer join remain unbound in the static trace. The table is therefore strong output evidence for this tested path, not proof of a universal writer implementation, other sample rates/layouts, or audio parity. Full JOC spatial rendering was not performed.
