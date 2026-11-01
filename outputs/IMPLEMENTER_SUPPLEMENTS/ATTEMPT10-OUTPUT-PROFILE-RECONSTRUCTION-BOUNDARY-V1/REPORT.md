# Attempt-10 output profile reconstruction boundary

## Result

The current hash-verified Rust source template emits a selected full-file job configuration with 1024 kbps, `file_position`, 1770-4 `measure_only`, DRC `none`, and preferred downmix `loro`. The hash-verified selected codec provider constrains the source plan to 48 kHz, 1024 kbps, 1536 samples, 4096 bytes, six blocks, `acmod=7`, LFE enabled, and five fullband channels.

For attempt-10, the job XML was reconstructed from that template and run-bound input/output paths; its XML digest was not captured by the app. The reconstructed prepared-config digest and run, executable, runner, and source-file hashes are sealed in the metadata package. However, the source snapshot was committed after the attempt-10 build, and no build-time source-tree digest ties the executable to these exact source bytes. Treat the profile as `SOURCE_SUPPORTED_RECONSTRUCTION`, not a run-captured config fact.

## Run outcome and scope

Attempt-10 is terminal `DONE/exit 0` and has a real whole-file hash mismatch against the user-designated keyless comparator. Both files are 17,395,712 bytes. The mismatch is confirmed for this run; this package does not prove the run binary used the reconstructed source template or explain the mismatch.

Input metadata (48 kHz / 38-channel / PCM24) does not itself prove output profile. The run receipt has no persisted job-XML/profile digest, and the source-to-binary link is not captured. The package contains metadata and hashes only; it contains no job XML, source code, or output/comparator bytes.
