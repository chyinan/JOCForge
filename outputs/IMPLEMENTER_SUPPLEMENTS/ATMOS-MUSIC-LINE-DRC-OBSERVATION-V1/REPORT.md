# Music Line DRC: bounded DEE observation

One authorized 38-channel ADM, 48 kHz/1024 kbps/5.1, same short window and all XML controls fixed except Line DRC; RF was film_light. Sequential film_light, music_standard and music_light jobs exited successfully. Each produced 32 parseable E-AC-3 access units / 49,152 decoded core samples per channel. The configured source range was 50,000 samples; this does not prove the exact final consumed window.

The research receipt reports `music_standard` had a different per-block dynrng code distribution from film_light. With an independent FFmpeg E-AC-3 core decoder, core PCM was byte-identical with DRC disabled (`drc_scale=0`), but differed in five non-LFE channels with `drc_scale=1`. For music_light, the inspected DRC fields and core PCM were identical to film_light on this one short window, under both decoder settings. The comparison was in memory and outputs were subsequently removed; Wash verified retained job XML/log/receipt identity but cannot independently replay the deleted files.

This is an observation, not the profile-to-dynrng algorithm or a general equivalence rule. No full JOC spatial render or listening test was done. Do not replace music_light by film_light generally or claim music modes implemented from this observation. No media, PCM, raw payload, source, decompilation or key is included.
