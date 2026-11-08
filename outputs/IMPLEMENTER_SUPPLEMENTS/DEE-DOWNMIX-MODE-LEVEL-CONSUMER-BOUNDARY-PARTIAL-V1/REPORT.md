# DDP downmix mode and level consumer boundary

## Scope

This Wash supplement records the static typed configuration mapping for preferred_downmix_mode and the four downmix mix-level fields, plus the first unresolved consumer boundaries. It does not establish that host DME values are written into the native DDP XML fields or that a selected level affects a matrix, BSI, PCM, or audible result.

## Mode path

The DME host model and child argv path are statically observed. The native DDP XML schema declares preferred_downmix_mode with default loro and four string tokens: loro, ltrt, ltrt-pl2, and not_indicated. The parser maps these tokens to U32 ordinals 0 through 3 and targets a typed MixMode object.

The setter callback is null and the writeback retains the pre-existing MixMode value. Thus XML token to effective stored mode remains UNBOUND. The host composite argv to native XML writer/parser input is also UNBOUND.

## Mix-level path

Each XML level field has a distinct typed MixLevel object with one U32 MixLevelValue. The static default for each is token -3 and internal code 4. Center paths have no extra code predicate; surround paths require unsigned U32 code greater than 2. Host keys, plugin XML keys, and a separate encoder-parser key family do not have a typed writer join.

The selected runtime owner/generation read or serialization for MixLevel remains UNBOUND_NOT_CAPTURED, and the first matrix or BSI consumer remains UNBOUND_NOT_TRACED.

## Handoff status

IMPLEMENTATION_READY=NO. Preserve the host-to-native writer break, effective MixMode value break, level same-owner consumer break, and matrix/BSI consumer break. Input-only token maps are not runtime results; runtime remains NOT_RUN. No DEE, WinDbg, A/B, media/PCM/payload/key/license read, or implementation change occurred.
