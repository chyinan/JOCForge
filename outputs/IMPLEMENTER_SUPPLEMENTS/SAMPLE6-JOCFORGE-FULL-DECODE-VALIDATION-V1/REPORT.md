# Sample 6 full output validation

The current authorized “人是猫” run completed with exit code 0 and published
4,247 access units. Its final `.ec3` is 17,395,712 bytes with SHA-256
`d9680298ac1d913f408b77a72b45171fc379dabb71de80e44c24d27cbf1de6da`.
FFprobe identifies E-AC-3 JOC / Dolby Atmos, 48 kHz, 1024 kb/s, 5.1(side),
135.904 seconds, and a 4096-byte first AU at position zero. A complete
FFmpeg decode-only pass to a null sink exited 0 with no errors. No PCM was
saved. This establishes a parseable and decodable full-track file.

Audio equivalence to a fresh DEE run is still NOT_RUN. Two concrete BSI
differences require same-input DEE measurement: the Rust output U5 is 17
(bit 45, width 5) while the existing same-input DEE receipt records U5=21;
and the Rust output's downmix mode/level fields are all zero while its model
job template records LoRo/-3 (bridge code 4). The Rust receipt does not expose
the selected LUFS input, and the DEE scalar's unit is unbound, so the U5
difference is not attributed to a specific formula or meter cause. No old
Python prototype or historical control-group payload was used.

This validates file structure and complete decode, not audio-essence equality,
Atmos spatial quality, or listening acceptance. The package contains neither
the encoded file nor PCM, keys, or license material.
