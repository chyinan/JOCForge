# Sample-6 source-time value correlation

The authorized 38-channel sample-6 source is now identity-bound by full streaming SHA-256 to the previously washed native reference. The same input hash has AXML programme start 3610 seconds, no BEXT time reference, no explicit AXML offset/FFOA attributes, and 6,522,906 samples/channel at 48 kHz. A previous hash-bound native receipt for this exact file reports offset=FFOA=3610 seconds.

Under the existing selected file-position FFOA-to-EOF formula, those exact resolved values give a relative start of sample 0 and an exclusive end at sample 6,522,906. This supports full-file sample-6 reading from sample 0.

The equality does not prove that AXML programme start wrote either native property. Static Ghidra evidence finds the programme-start parser and a separate numeric offset property read/serialization path, but their typed producer join remains open. No first reader request was captured. Do not generalize this sample-specific result into an offset/FFOA resolver for other ADMs.

The R1 source package checker passed in normal and optimized Python modes. This Wash package contains no source code, binary, PCM payload, private key, or absolute path.
