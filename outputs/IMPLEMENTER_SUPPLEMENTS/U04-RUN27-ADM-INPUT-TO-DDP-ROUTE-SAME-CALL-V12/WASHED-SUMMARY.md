# Run27 ADM input lookup and DDP route: capture files separated

Disposition: NATIVE_PATH_DRIVEN_DDP_ROUTE_OBSERVED; READER AND SOURCE IDENTITY OPEN. This v12 Wash package clarifies which capture file belongs to run27 and keeps older captures separate.

## Provenance

Source v12 manifest SHA-256: 7b0bdaae6cf2825bf36e229cdca9372dbbe72da3c1c7cd50b033e12f841a60d0.
Source EVIDENCE.json SHA-256: 867507d27937e53ac8720e84099dac9315019e661e60981e0eea15cf7f9ba161.
Run27 same-invocation capture SHA-256: 50827ff605a22ce3f5b2393a98a18b30ac46d63168b32f06cc2c26150fe7ed99.
Separate run24 capture SHA-256: deb407163796774f199ce55000865676cecc77fccfd04074ba3c3757130cddd4.
Separate run26 capture SHA-256: 360d3414609829e8e23587950fc2110dff6d150b1f741a84340cf640e0ac1138.
The source v12 verifier passed normal and optimized Python; member hashes were independently rechecked.

## Run27 only

In run27, one ADM pathname target resolved. Its derived stream_object property lookup returned false and left output unset; RunAudioFilter's collected list was empty. The same thread nevertheless reached DDP routing with 38 frame channels: ten numeric route codes to slots 0..9 and 28 additional indices to slots 10..37. The reference encoding completed at 262144 bytes with SHA-256 077977564a8da534a52dee3feb1c791cc0825dd9cd7eb975575799b02450e686. No PCM values were read.

The run27 job XML was 739f0053a4eb87dfc99c3d8111d3b243aa38d07cf444c1e4ae87300f1687f193, not the current accepted XML 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9.

## Prior runs are separate

SEPARATE-RUN24-ROUTING-CAPTURE.json and SEPARATE-RUN26-INPUT-PROPERTY-CAPTURE.json are separate historical observations, not part of run27 and not joined for generation claims. The run24 route capture is included only as history. v12 adds no new capture values beyond the run27 package already washed in v11.

The generic StageRunner stream_object transfer was not used for the run27 ADM invocation. This does not identify which component opened/decoded the pathname and populated DDP frame pointers. ADM reader/decoder identity, frame slot to CHNA/physical/global source, and frame slot to selector column remain UNBOUND. The v8 direct AtmosMezz Run property-write attribution remains withdrawn.

No raw audio, source code, decompilation, addresses, license, or key is included. This is a DEE reference observation, not a JOCForge product output or full-chain acceptance.
