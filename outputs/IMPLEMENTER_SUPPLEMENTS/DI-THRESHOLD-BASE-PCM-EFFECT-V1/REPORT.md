# Dialogue-intelligence threshold changes base PCM for one matched DEE pair

## Observed effect

On one authorized ADM with `dialogue_intelligence=true`, changing only `speech_threshold` from 15 to 100 changes the base 5.1 PCM produced by decoding the two DEE E-AC-3 outputs with the OpenJOC v0.20.1 internal raw E-AC-3/JOC decoder. The two streams align over all 4,247 AUs and 6,523,392 samples per channel; every one of FL/FR/FC/LFE/Ls/Rs has nonzero RMSE and max error. The largest RMSE is Ls; the smallest is LFE.

The threshold settings also changed DEE measurement gating (`dialogue` to `level`) and integrated loudness. This is direct evidence that the threshold option affects audio-bearing output for this matched sample, not merely a UI label. It is still a single DEE input pair.

## Boundaries

The comparison used OpenJOC's v0.20.1 base compatibility PCM decoder with a temporary CLI comparator built from the v0.20.1 tag; the decoder module itself retained its tag hash. It did not use FFmpeg/FFprobe, did not invoke the speaker/Atmos renderer, and saved no PCM. The temporary comparator and build tree were removed. This does not prove Dolby Atmos renderer parity or that JOCForge has implemented the same threshold behavior.

For implementation, the next useful step is to compute the same threshold-15/100 measurements and output through JOCForge on this ADM, then compare the decoded base PCM to the two DEE outputs. Repeat on another authorized ADM before generalizing.

The supplement contains only scalar metrics, hashes, and provenance. It contains no E-AC-3 bytes, PCM, temporary source, key, or license.
