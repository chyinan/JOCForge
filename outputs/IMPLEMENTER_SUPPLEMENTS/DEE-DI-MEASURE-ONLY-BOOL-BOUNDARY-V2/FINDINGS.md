# Dialogue Intelligence measurement boundary V2

The conditional static path can place a Bool candidate at measurement setup +0x94. The first per-record consumer, FUN_1801764e0, is scoped to reads of setup +0x9c and +0x140. Its first direct helper, FUN_180182850, receives a per-record source pointer and four pointer arguments and copies those inputs to a record-like object; no direct setup pointer or +0x94 field is passed on that edge. The typed owner relation to the Bool therefore remains UNBOUND.

The F06 aggregate uses per-record F32 at record+0x14; no +0x94-to-result dependency is bound. Prior measure-only A/B receipts show sample-dependent gating/diagnostic changes, but the unit was not stated and those observations were reused, not rerun.

No new DEE/Ghidra query or runtime capture was performed. The Bool algorithm use, owner/generation, F06 result relation, and audio/JOC effect remain UNBOUND/NOT_RUN.
