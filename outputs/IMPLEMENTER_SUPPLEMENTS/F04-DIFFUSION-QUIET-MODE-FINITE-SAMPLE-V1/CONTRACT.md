# F04 quiet/mode finite sample evidence

Research report: three authorized ADM excerpts plus one secondary reference, each four AUs at 48 kHz/1024 kbps, produced 5.1 E-AC-3 output recognized by ffprobe and decoded to null by ffmpeg. This tests the native reference, not JOCForge. Spatial rendering, listening, full song and product acceptance are NOT_RUN.

For the reported native quiet boundary, abs(R32(window*sample)) > 2^-23 is strict. Four synthetic threshold/mute cases matched on each sample. Raw current history is copied; mute clears the window only when a strict-threshold witness exists. Local reset observations do not establish whole-programme reset. F04FM reported lane [0,0,-1,1,1,2,2,3,3,0,0] and mode [1,-1,0,1,-1,1,-1,1,-1,1,-1], 11/11 each matching approved generator. Do not extrapolate to other layouts.
