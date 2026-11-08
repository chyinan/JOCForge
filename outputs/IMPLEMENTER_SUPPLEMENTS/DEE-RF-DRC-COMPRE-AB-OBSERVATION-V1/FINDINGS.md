# RF DRC profile A/B observation

A single N38, 48 kHz, 24-bit, 1024 kbps controlled pair changed only rf_mode_drc_profile: film_light versus music_standard. Both DEE runs exited naturally with code 0. The two outputs had equal byte length but different hashes.

The bounded parser summary observed 32 frames and 192 audio blocks in each output. compre was present in all frames. Its code distribution and sequence hash differed between A and B; the dynrng code distribution and sequence hash matched. No raw BSI/frame bytes or PCM were persisted.

This establishes an RF-profile-associated BSI compre difference for this one N38 input/window. It does not establish decoded audio or loudness/gain behavior, the meaning of every code, or support/equivalence for the other four profiles, line-mode profiles, N36, or other rates.
