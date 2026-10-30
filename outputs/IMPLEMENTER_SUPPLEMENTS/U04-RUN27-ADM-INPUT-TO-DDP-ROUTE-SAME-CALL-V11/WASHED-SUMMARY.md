# Run27 ADM input lookup to DDP route in one reference invocation

Disposition: NATIVE_PATH_DRIVEN_DDP_ROUTE_OBSERVED; READER_AND_SOURCE_IDENTITY_UNBOUND. This is one successful DEE reference invocation, not the current JOCForge product chain.

## Provenance and verification

Source v11 manifest SHA-256: 25dc9ac778206b07eedaabe0c49fbb09075752ab92fad653902da694727e6ee9.
Source EVIDENCE.json SHA-256: 79d7c10e903833fe69a5544cf515fce196937acbd0b41034bd949e067bbc6697.
NATIVE-ADM-TARGET-LOOKUP-CAPTURE.json SHA-256: 50827ff605a22ce3f5b2393a98a18b30ac46d63168b32f06cc2c26150fe7ed99.
Normal and optimized source verifiers passed; Wash rechecked all manifest members. No audio samples or source material were included.

## Same-invocation observation

Run27 resolved exactly one ADM pathname target. RunInput queried its derived stream_object property; the lookup returned false and left the output unset. RunAudioFilter's collected stream_object list had zero entries. The same thread nevertheless reached DDP route consumption with 38 frame channels. Ten numeric category entries routed to pointer slots 0..9, and 28 additional record indices routed to slots 10..37. The reference encoder completed and emitted 262144 bytes with SHA-256 077977564a8da534a52dee3feb1c791cc0825dd9cd7eb975575799b02450e686. No PCM samples were read.

The reference used BWF hash 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1 and job XML 739f0053a4eb87dfc99c3d8111d3b243aa38d07cf444c1e4ae87300f1687f193. This differs from the currently accepted job XML 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9.

## What remains unbound

The generic StageRunner stream_object list did not carry this ADM input. The DDP frame route therefore used a separate pathname-driven source route, but the reader/decoder that opens the file and constructs DDP frame pointers is not identified. DDP pointer slots remain unbound to CHNA AudioTrackUID, physical track, F03R global source ID, and selector columns. Numeric category meanings are also unbound. Do not infer any join from slot number, channel count, or matching reference output hash.

This corrects the earlier unsupported claim that AtmosMezz Run directly writes stream_object; that attribution remains withdrawn. This package contains no source code, decompilation, addresses, module path, license, key, or raw audio.

## Capture file separation

Run27 same-invocation target lookup and 38-slot routing are in NATIVE-ADM-TARGET-LOOKUP-CAPTURE.json (source SHA-256 50827ff605a22ce3f5b2393a98a18b30ac46d63168b32f06cc2c26150fe7ed99). SEPARATE-RUN24-ROUTING-CAPTURE.json (source SHA-256 deb407163796774f199ce55000865676cecc77fccfd04074ba3c3757130cddd4) is a distinct earlier run24 capture, included only for history; it is not evidence from run27.