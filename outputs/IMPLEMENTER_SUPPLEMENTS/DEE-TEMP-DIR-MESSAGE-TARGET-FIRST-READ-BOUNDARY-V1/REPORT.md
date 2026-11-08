# Temp Message target first-read boundary

Reference-only Wash supplement. The reused receiver receipt binds `FUN_14006c760`'s RCX to the returned `dee::Messages::Message` object. The new Ghidra entry-prefix query's first non-stack read is a four-byte load from receiver offset `+0x48`.

The query stops at this first read; it does not inspect later instructions or branches. The field meaning is unknown. No typed path joins this field to path-object `clean_temp+0x90`; file cleanup behavior and runtime owner/generation remain `UNBOUND`. `IMPLEMENTATION_READY=NO`.
