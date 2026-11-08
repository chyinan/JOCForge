# Line DRC N38 BSI observation

A single-factor N38 pair held the authorized input, RF=film_light, rate, time base, and time window constant. The only normalized XML difference was line_mode_drc_profile: film_light (A) versus none (B). Both DEE arms exited 0 and produced 32-frame/192-block EC-3 outputs.

The sealed BSI probe, run in memory on each output, reported identical compre presence, code distribution, and sequence. The dynrng distribution changed from A's {0x00:50, 0x01:65, 0x02:45, 0x03:32} to B's 0x00 for all 192 blocks; the dynrng sequence hashes differ. Output SHA-256 values differ.

This is a bounded observed BSI difference for one N38 input/window. The probe does not identify the typed DDP parameter 0x0d setter, and no decoded PCM, gain, listening, or audio/JOC effect was measured. No generalization to other inputs, profiles, or rates is claimed.
