# PCM24 reader F32 normalization rule

## Rule

For the hash-bound selected 38-channel PCM24 input, the static reader caller supplies context value 7. In that context, the three-byte-per-sample branch sign-extends packed little-endian PCM24, shifts the signed integer left by 8, converts to binary32, and multiplies by binary32 2^-31.

This is exactly signed_pcm24 / 2^23. The intermediate shifted integer has at most 24 significant bits and is exactly representable in binary32; division/multiplication by a power of two also produces an exact binary32 value for the full signed PCM24 range. The result spans [-1, 1 - 2^-23].

The Wash decision approves this formula for the PCM24 source-reader conversion under context 7 and bytes_per_sample 3. The earlier S16 branch is a separate context 4 and does not apply here.

## Boundary

The source package uses Ghidra static evidence only; no native sample-6 job or PCM values were captured. The first downstream numeric F32 consumer and the same-job reader-row to DDP-row owner/generation join remain open. This supplement approves the numeric conversion at the reader boundary, not the whole audio chain.

## Provenance

Source package manifest SHA-256: fbe36b291be5dd965a94c55350ce9981ef05250f3aa3dc12b057786adf26f9cd
Source package seal SHA-256: f77f2e0fd951d8252635233b320f04138b49120261a17c3070dbb5b76f18d374
The source package checker passed in normal and optimized Python modes. This supplement contains no source code, raw disassembly, binary, PCM payload, private key, or absolute path.
