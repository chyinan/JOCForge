# temp_dir clean flag first-consumer boundary

On the known StageRunner resolver path, the same path_object written by the clean_temp setter reaches FUN_14007b6f0. Its first scoped read compares one byte at path_object+0x90, but only when path_object+0x68 is non-null. The branch selects local path-string assembly and passes the assembled value through the provider object at +0x68, vtable slot +0x58.

This establishes a path-construction use, not file deletion or cleanup policy. The +0x68-null Message path is separate and does not join Message+0x48 to clean_temp. Provider target/type, runtime owner/generation, and cleanup/delete effect remain UNBOUND/NOT_CAPTURED. No global absence claim is made.
