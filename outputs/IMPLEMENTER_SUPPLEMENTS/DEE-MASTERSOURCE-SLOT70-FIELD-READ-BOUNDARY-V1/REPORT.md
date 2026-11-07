# MasterSource slot+0x70 field-read boundary

Reference-only Wash supplement. It identifies the getter's two QWORD reads, not duration/sample semantics. `IMPLEMENTATION_READY=NO` for range behavior.

The constructor-fixed `MasterSourceImpl` vtable `+0x70` target is `FUN_1805364e0`. Its complete 10-byte body reads a QWORD pointer from `this+0x30`, then reads a QWORD at pointee `+0x50` and returns it. It does not read `this+0x20` or `this+0x28`, and makes no direct calls.

The pointee's owner/generation and the returned QWORD's type/unit are unbound. This does not establish a duration, start/end sample range, PTS, or audio effect.