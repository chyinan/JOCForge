# Wash review: AtmosMezz helper ADM BWF path (V2)

Ghidra static evidence shows that `dee::input::AtmosMezz::Run` delegates its provider-supplied input to the hash-bound `atmos_info.exe --input` helper. The helper has a type-specific ADM BWF factory and parser path; that path validates `chna`/`axml`, parses ADM XML, and constructs `atmos_storage_master_s`. Thus the helper can process ADM BWF metadata; it is not restricted to a different container.

This is a capability result only. The existing full-file sample-6 receipt confirms a completed `atmos_mezz/file_position` DEE job, but contains no helper PID, selected factory callback, master/storage object or vtable. It does not prove which helper path that run selected, nor that helper and DACT share a runtime pointer/object. The minimum discriminator is a single same-job/generation observation immediately after the helper's type-specific BWF master open returns.

No audio algorithm or field value is supplied. This package is limited to input-parser capability and runtime route identification. FFmpeg/OpenJOC remain posthoc or public-syntax checks only.
