# R138 context-mode and BSI-owner condition boundary

## Typed constructor path

Ghidra traces the component context mode from a source member at `+0x54` as `(member == 1)`. The new context is zero-initialized; its update flag at `+0x854` begins at zero. On successful mode-one initialization the flag becomes one; mode zero leaves it zero. These constructor outcomes are therefore `(mode, update_flag) = (0,0)` or `(1,1)` for the traced path.

The parent fixed-block writer evidence says both successful branches write zero to the low U16 at owner `+0x34`, selecting different conditional BSI writers. This is a conditional rule for those constructor branches, not a captured R138 value or allocation-time value before the setter.

## R138 profile boundary

The hash-bound R138 profile has 1024 kbps, embedded-timecode window 86640–86664, measure-only 1770-4, dialogue intelligence off, DRC none, and no custom DialNorm element. The typed path does not connect its XML fields to component source member `+0x54`. Therefore R138 mode, update flag, owner U16, and zero applicability remain `UNBOUND`. The prior `(0,0)` runtime observation used a different job hash and is not transferred.

## Disposition

Reference only; `implementation_ready=false`. Keep the mode producer and conditional zero behavior scoped to the traced constructor, but do not apply it to the R138 profile until the XML-to-`+0x54` edge is established. No new DEE run or U16 capture occurred; no audio, payload, key, or license content was read.
