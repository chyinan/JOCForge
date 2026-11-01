# Wash review: keyless full-song candidate and DEE reference

Confirmed on disk: the approved full-song `expected.ec3` fixture and R3's hash-bound DEE full-file receipt both carry SHA-256 `34d18a6f...` for the same user-authorized sample-6 input SHA `7826dcb...`. Historical user/session evidence says the Python model output and DEE reference were byte-identical at that hash. The approved fixture is marked validation-only.

There is also a separate keyless research candidate, SHA-256 `383d7de4...`, with the same 17,395,712-byte size but a different bitstream (first byte difference at 1411, AU 0). Its report says it uses public CRC32/CRC8 rather than the private HMAC key. A decode-only comparison reports identical base PCM SHA across all six channels and zero scalar differences. OpenJOC metadata reports JOC in 4247/4247 AUs, but strict OAMD body validation was not passed and renderer/vendor fidelity or user playback is not claimed.

This supports the user's keyless-MVP intuition: a keyless candidate can decode identically to the DEE reference while its EC3 bytes differ. It does not establish that simply deleting a private-HMAC block is sufficient, does not expose the Python implementation, and is not a Rust output. The usable role is a validation oracle after JOCForge writes its own file; the Python source and candidate media are not included here.

FFmpeg and OpenJOC results are posthoc validation only. They must not supply DEE audio algorithms, field values, or defaults.
