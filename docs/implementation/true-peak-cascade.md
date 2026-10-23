# F06TP selected true-peak block cascade

This direct slice processes one caller-supplied six-channel 1024-sample block using the hash-bound authorized 6+5 binary32 FIR taps. It performs the specified two-stage 4x cascade for L/R/C/Ls/Rs, bypasses interpolation for logical LFE ordinal 3, and returns both measurement samples and a complete next-state proposal containing all six 4628-word histories plus the twelve programme maxima.

All FIR multiply/add operations use separately rounded binary32 arithmetic in the specified order; no FMA, host resampler, or double accumulation is used. The local F06L.4/.5 sample/true-peak metric leaves are included because the F06TP block vectors bind those outputs; other log, energy, loudness, integrated-gate, or meter functions are not included.

The local tests independently reconstruct each of the nine input recipes, verify source identity, process the eight-block continuous sequence from actual returned state, and compare each lane's complete measurement digest, complete next-state bytes, peak projection, and programme maxima. The ninth case resets and seeds only the LFE history to verify exact bypass preservation.

The base package explicitly classifies the FIR words as authorized compiled functional constants for internal implementation only and says public or third-party distribution is not authorized. This local implementation and its parameter files are uncommitted and must not be committed or pushed to the public branch without separate authorization.

This does not implement audio acquisition, the shared measurement ring/window clock, F06W K-weighting, none-DRC composition, ID1 timer/FIFO/EMDF, source/AU accounting, parent transaction, real ADM/render, or product listening acceptance.
