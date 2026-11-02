# DEE DI speech-threshold matched-control result

## Result

On the same authorized ADM, with `dialogue_intelligence=true` held constant, changing only `measure_only/speech_threshold` from 15 to 100 changed the reported measurement gate from `dialogue` to `level` and final integrated loudness from -25.022484 to -16.784912 (measured loudness -25.02 to -16.78). Speech percentage, dialogue loudness, ITU loudness, loudness range, sample peak, and true peak were unchanged.

Both runs completed measurement and produced 17,395,712-byte E-AC-3 files with different SHA-256 values. No PCM was extracted or compared. The output hash difference therefore does not prove a decoded audio difference or that the threshold changes the encoded sound.

## Use for implementation

This closes an observed DEE measurement-control behavior for one matched ADM/job pair. It is suitable for implementing the threshold's measurement/gating semantics and checking the same scalar results on multiple authorized ADM files. It does not yet authorize an audio-DSP change or prove a listening effect. Keep threshold and dialogue-intelligence controls explicit in JOCForge's XML mapping.

## Limits

Two sandboxed pairs crashed before measurement; the isolated `require_escalated` pair completed with status 0. The supplement contains scalar results, identities, and output hashes only; it omits E-AC-3 bytes, raw logs, PCM, key, and license. No implementation source was changed.
