# 36-track source allocation: two native excerpts

Research-side metadata/static and native observations on two authorized 36-track ADM BWF excerpts report 11 prefix sources plus 35 regular sources: 46 active sources. Each of eight callbacks has 35 regular writer rows targeting slots 11..45; the one semantic LFE is excluded. The two sample files happen to put LFE at physical slot 3; this is not a general slot rule.

Native audio storage allocated 47 current/window slots; slot 46 was zeroed but lies outside the 46 active sources and was not a writer target. Its control metadata was not read. Do not invent a 36-to-48 padding rule or assume unused metadata. The observed native capacity is not by itself a requirement for Rust to expose 47 logical sources.

Later JOC input is a different 16-row domain. For the two short excerpts, reference output was reported as four 4096-byte AUs, recognized as 48k/1024k/5.1 DD+ Atmos and decoded to null by ffmpeg. These are native reference samples; 36-track model replay, JOCForge output, full song, spatial render, listening and general ADM support remain NOT_RUN.
