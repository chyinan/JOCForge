# Music Line DRC: Love Song observation

One authorized 36-channel Love Song ADM, 48 kHz, 1024 kbps, 5.1; same 230–235 second source window and XML controls, except Line DRC. RF was film_light, DI true and threshold 15. All three DEE jobs exited successfully. The retained outputs each contain 157 frames / 942 audio blocks and decode to 241,152 5.1 core samples per channel.

Wash independently checked the three normalized XML jobs, retained outputs and exit receipts; reran the supplied E-AC-3 DRC field parser on each retained stream; and decoded core audio in memory using FFmpeg, separately at drc_scale 0 and 1. The film_light and music_light dynrng sequences match; music_standard differs. At scale 0, all three core decodes are byte-identical. At scale 1, music_light matches film_light; music_standard differs. No decoded PCM was persisted. The DEE outputs remain in the research workspace and are not included in this package.

This is an observation on one five-second window, not a general equivalence between profiles or a specification of profile-to-dynrng curve/state/rounding. Full JOC object/spatial rendering and listening tests are NOT_RUN. No media, PCM, raw payload, source, decompilation or key is included.
