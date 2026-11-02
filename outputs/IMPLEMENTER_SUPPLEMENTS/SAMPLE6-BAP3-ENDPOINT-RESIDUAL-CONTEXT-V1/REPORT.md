# Human Cat BAP-3 endpoint candidate: residual context

The native E-AC-3 BAP-3 scalar rule has a concrete endpoint split from the current washed F12S rule. For coefficient `+1.0` (F32 bits `0x3f800000`), exponent `0`, and BAP `3`, the native path computes `trunc((x + 1) * 3.5) = 7`, clamps to level `6`, and packs `110`. The current washed rule treats raw `7` as a reserved sentinel and rejects the candidate.

The split is relevant to a matched Human Cat comparison window, but is not a demonstrated cause. At decoder DRC scale 0 in AU interval `[2750, 2875)`, the inherited per-channel metrics report decoded channel index 3 RMSE `0.0054540887`, while indices 0, 1, 2, 4, and 5 are each at or below `0.0004557199`. The 5.1(side) index-to-LFE assignment is layout-derived; no BAP/coefficient rows for this sample window were captured. Raw-7 occurrence and PCM causality remain `UNOBSERVED`/`UNBOUND`.

The input-only vector is suitable for the existing Rust quantizer seam. Run it first. If the current implementation rejects it, correcting the endpoint to native saturation is a narrow algorithm fix; only a matched output comparison can establish whether it improves this sample. Keep the DRC controls and window-table inputs fixed in that comparison.

No PCM, encoded payload, private key, license material, disassembly, or native source code is included. The inherited residual metrics are reference-linked observations and were not recomputed by this addendum.
