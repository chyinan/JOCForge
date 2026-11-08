# MP4/DASH output-selection boundary

The reused Ghidra 12.2 receipts bind a static setter candidate that accepts mp4/dash and stores the string at receiver+0x3b. Mp4::Run calls InitPlugin with the same param_1; InitPlugin reads output_format at +0x1d8 and passes a property array through an indirect interface slot +0x18.

The setter receiver has not been joined to the selected Run/InitPlugin instance. The +0x18 target, ABI, registration owner, and selected muxer/writer remain UNBOUND. A separate DDP FileWriter/fwrite sink is not evidence of an MP4 muxer.

The XSD is not installation-bound, and exact job_config XML bytes are unavailable. No DEE run, patcher run, or MP4/DASH output exists in this evidence. Tokens and vectors remain NOT_RUN. This package is reference-only; it does not enable output-container support.
