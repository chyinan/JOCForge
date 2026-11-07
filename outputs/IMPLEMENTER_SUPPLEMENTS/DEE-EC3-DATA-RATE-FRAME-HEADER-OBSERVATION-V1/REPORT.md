# DEE 1024/768 kbps E-AC-3 frame-header observation

A successful ordinary DEE CLI 768 kbps run was compared with the retained, successful 1024 kbps attempt-05 baseline. Both are the same authorized My Jealousy N36 input, 48 kHz, 5.1 Atmos DDP, same 0–24 frame window and M4A output container; `data_rate` is the only semantic XML difference.

Wash independently streamed the E-AC-3 track from each retained M4A through a copy-only demux pipe and parsed/discarded frame bodies after each syncframe header; no re-encoding occurred and no packet bytes were saved. Each stream had 33 syncframes. The 1024 kbps baseline had frame-size code 2047 and 4096 bytes per syncframe; the 768 kbps run had code 1535 and 3072 bytes per syncframe. Both matched the earlier MediaInfo/ffprobe packet-size summaries.

This is direct evidence for two successful rate settings on this input/window. It does not identify the DEE carrier writer implementation, prove the six-rate range, or close all M4A timing/container details. No PCM or raw encoded payload was persisted and no key/license was used.
