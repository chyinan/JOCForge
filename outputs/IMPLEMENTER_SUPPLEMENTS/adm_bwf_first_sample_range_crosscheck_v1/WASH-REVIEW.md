# ADM BWF first-sample range cross-check

## Scoped finding

A hash-bound Ghidra package binds offset and FFOA reads to one source-input property member in one run context. The static explicit FFOA-start rule is:

delta seconds = max(source FFOA seconds, offset seconds) - offset seconds
start sample = round(sample rate * max(0, FFOA - offset))

The sample count N is an exclusive EOF endpoint. The range constructor rejects start >= N and end > N; it does not saturate. The precise sample-rounding tie rule is not established here.

## Input cross-check and limits

The authorized First Love BWF metadata is PCM24, 36 channels, 48 kHz, with 12,361,093 samples/channel. CHNA contains 36 tracks and UIDs. AXML has programme start/end 3610.0 / 3867.52277 seconds; BEXT and explicit offset/FFOA attributes are absent. A 576-byte DBMD chunk was not parsed. The full file hash was not computed; metadata was read and the entire PCM data payload was skipped.

This is a 36-channel metadata cross-check, not the selected 38-channel output profile. The AXML programme start is not asserted to be the resolved DEE offset or FFOA. Their auto resolvers and the actual first reader request remain unbound. On the whole-file default path explicit range arguments are suppressed, so this explicit-range formula does not itself prove the default reader cursor.

## Wash disposition

This package is a conditional static time-range fact, not an implementation-ready source-start rule. Do not change the selected profile source cursor from it without a same-file/attempt observation of resolved offset, FFOA, and the first reader request.

## Provenance

Source package manifest SHA-256: b299d17fcff4f177471d564424a84f61d6efd34bc6c30ec86ec36b41474bae14
Parent checker passed in normal and optimized Python modes. The Wash package contains no raw source code, disassembly, binary, PCM payload, private key, or absolute source path.
