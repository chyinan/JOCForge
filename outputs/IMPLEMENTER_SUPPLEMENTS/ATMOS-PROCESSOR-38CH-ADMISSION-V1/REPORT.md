# AtmosProcessor 38-channel admission and frame-capacity guard

## Static guard contract

In the hash-bound `dee_audio_filter_ddp_atmos.dll` image, `AtmosProcessor::process` (`FUN_18018ae20`, RVA `0x18ae20`) checks the input descriptor's U64 `channel_count` at descriptor byte `+0x08` against the configured count in processor state slot 4. It also requires descriptor `frame_count` at `+0x10` not to exceed processor capacity in state slot 0.

The configured count is taken from the master descriptor's U16 at `+0x0A`: `FUN_180186870` stores that value in the descriptor's channel_count, and `FUN_18018a750` copies the same U16 into processor state slot 4. The guard has no hard-coded 36-channel maximum; a 38-channel descriptor passes if that particular config instance carries 38 at `+0x0A`.

The frame path accepts 0, 512, or 1536 samples at the relevant initializers. The selected 48 kHz target profile uses 1536 samples per access unit, which is within the 1536 capacity when initialized accordingly. Flush size 0 requires a prior valid 512/1536 capacity initialization.

## Scope for 人是猫

The authorized `人是猫` ADM metadata is 38 channels / 38 CHNA rows / 48 kHz / 24-bit (input SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`). Static evidence supports feeding 38 row pointers and a 38-channel descriptor through this guard when the master descriptor is configured to 38. No runtime instance was observed, so the exact descriptor `+0x0A` value remains unbound. This package does not establish that the overall JOC job or downstream renderer accepts the 38-channel source; it isolates the AtmosProcessor guard only.

## Recommended implementation check

Keep the actual parsed channel count, set both the descriptor count and its master/config count from the same 38-channel ADM input, and use the selected frame size 1536. Do not pad 36 to 38 or claim an observed run. This input-admission rule is ready for a direct trial; the remaining field-value receipt can be captured from the run receipt without reading PCM.

## Exclusions

Ghidra 12.2 static analysis only. No BWF payload was opened, no PCM read, no DEE/converter run, and no private key/license read. No binary, source code, or decompilation is included.
