# Wash correction: external DialNorm reference is not DEE behavior

This addendum narrows the earlier FFmpeg citation. FFmpeg is a third-party parser/decoder implementation. Its E-AC-3 DialNorm handling and gain application do not prove that DEE emits, interprets, or audibly applies the field the same way.

The only DEE field mapping currently established is Ghidra serializer order: record +8 is BSID; record +0x20 is DialNorm by header order. R141 did not capture the +0x20 value, its upstream source at owner +0x34 remains UNBOUND, and no DEE playback/render test was done. The FFmpeg reference is retained only as an external syntax/decoder cross-check, never as an algorithm, formula, default value, or behavior source for JOCForge.

All audio-affecting behavior remains governed by DEE evidence and the approved codec behavior; this package does not authorize deviation from that target.
