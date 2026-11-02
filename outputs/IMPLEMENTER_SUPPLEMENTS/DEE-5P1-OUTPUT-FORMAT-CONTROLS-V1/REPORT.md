# DEE 5.1 output-format control boundary

For the hash-bound authorized job XML, the selected profile explicitly requests MP4 output, 1024 kbps, Lo/Ro, DRC `none`, and a file-position range. It contains no fields for sample rate, `acmod`, `lfeon`, channel count, or channel order. The same job's existing output metadata reports E-AC-3, 48 kHz, six channels, `5.1(side)`, and 1,024,000 bit/s. Rust's selected profile reports the same high-level stream metadata, while using an internal mantissa-lane order that is not a proven wire-channel order. MP4/M4A versus raw EC-3 is container packaging, not audio essence parity.

The selected native writer path is statically bound through a frame-provider buffer and byte span into a `FileWriter`/`fwrite` sink. The job XML is not joined to that native configuration object, and the frame provider's sample-rate/acmod/lfeon/channel-order producer remains unbound. The package therefore does not show a frame-control mismatch and does not prove frame-level equivalence.

Use this boundary to keep the matching high-level controls and avoid attributing the measured PCM residual to the container difference. Do not infer DEE's absent frame fields from Rust, channel layout, or internal lane order. No code change to audio parameters is justified by this evidence alone; compare a same-window decode if frame-control suspicion remains.

No DEE run, PCM/payload/key/license read, or implementation contact occurred in this slice. The included derivative omits the Ghidra script and all source/decompilation material.
