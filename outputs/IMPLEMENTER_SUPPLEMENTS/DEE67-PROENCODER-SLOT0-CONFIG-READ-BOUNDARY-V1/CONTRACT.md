# ProEncoder slot-0 Config+0x58 read boundary

## Wash-approved scope

For the exact hash-bound user-supplied candidate image, DDPJocProEncoder vtable slot 0 at base `0x140b36290` points to `FUN_1402ac530` (`.pdata` extent `[0x1402ac530,0x1402ac572)`). A bounded 19-instruction scan of this method found no P-code LOAD resolving to same-object `this+0x58` and no unresolved-base `+0x58` load candidate.

## Implementation boundary

This is a slot-0 method result only; it does not establish global absence or close `Config+0x58` to `profile_index=1`. Other methods and the profile-source/consumer join remain `UNBOUND`. The candidate build is user-supplied and hash-bound for this observation; official-version identity is not established. No acoustic rule, media, PCM, or payload is included.
