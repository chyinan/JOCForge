# DEE LFE descriptor and recurrence-branch observation

**Status:** lane-5 LFE branch and recurrence argument captured in both authorized 36-row and 38-row runs; sample-scoped only.

## Result

In each run, the layout state was ID 7 with five full-bandwidth lanes and six total lanes. The parent lane loop selected lane 5 as LFE, mapped it to descriptor slot 5, and passed its selected-row U16 stride as **1**. The recurrence call for that same lane separately received a **0** argument, selecting the recurrence branch. The descriptor stride and recurrence selector are distinct fields and must not be equated.

Ghidra 12.2 identifies the plugin function at `dee_audio_filter_ddp_atmos.dll+0x229e30` as having the same short-selector/nonzero-helper/64-iteration recurrence shape and 184-byte body as the standalone encoder candidate. This is a code-shape comparison, not a cross-image owner join or exact algorithm-parity proof.

The 36-row and 38-row captures use different ADM hashes, config hashes, processes, state objects, descriptor tokens, and temporary buffers. Equal stride/branch values across those runs do not establish shared identity, ownership, or generation. No audio samples or buffer contents were read.

## Output evidence

Both DEE jobs completed with MediaInfo reporting E-AC-3 JOC, 48 kHz, 1,024 kb/s CBR, six-channel 5.1, and an LFE channel. The 36-row output was played by the user in Dolby Reference Player 3.2.0; it decoded/played through a two-channel endpoint, and the player identified Atmos/JOC 5.1 content. Physical 5.1/Atmos speaker output, full-duration playback, and 38-row player playback remain **NOT_RUN**. EC-3 payloads and output hashes are not included in this package.

This evidence lets the implementation compare its LFE row stride and recurrence branch separately against DEE for these two samples. It does not establish support for every ADM layout/rate, JOC renderer parity, or a universal DSP parameter set.

## Verification and privacy scope

The source package's manifest and seal were hash-checked; its verifier passed normally and under `python -O`. The sanitized supplement excludes Ghidra scripts/raw disassembly, native binaries, PCM, encoded payloads, process addresses/tokens, license material, and private keys. No bit-exact comparison was performed or required.
