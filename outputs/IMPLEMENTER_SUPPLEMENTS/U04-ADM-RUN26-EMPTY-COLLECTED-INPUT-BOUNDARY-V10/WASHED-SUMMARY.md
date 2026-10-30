# Run26 ADM input-property path boundary

Disposition: NATIVE_INPUT_PROPERTY_OBSERVED; ADM decoder and source-slot identity remain open. This update supplements the separate run24 DDP route capture. It does not join the two runs or close global source identity.

## Provenance

v10 source package manifest SHA-256: d4c031a263cd486f1799d764a2ed57f9fdd6b88c9e92cbeb2d131bc097021e69.
v10 EVIDENCE.json SHA-256: dbeb069c7b7ccc18cda240dce2c7acf7344a3c138f1d480a6a3686202d093220.
NATIVE-INPUT-PROPERTY-CAPTURE.json SHA-256: 360d3414609829e8e23587950fc2110dff6d150b1f741a84340cf640e0ac1138.
Separate run24 route capture SHA-256: deb407163796774f199ce55000865676cecc77fccfd04074ba3c3757130cddd4.
The v10 checker passed normal and optimized Python; Wash rechecked the source member hashes. No raw audio, feature values, addresses, source, license, or key are included.

## Run26 same-invocation observation

Run26 reached RunAudioFilter and then DDP frame processing on the same thread. At RunAudioFilter entry, the collected stream_object list had zero entries; it remained empty while DDP processing was reached. The dee_audio_filter_source property held a pathname string matching the authorized ADM fixture, with the full path omitted. The reference encoder completed a 262144-byte EC3 whose digest matched the prior reference artifact. No audio samples were read.

Static StageRunner logic queries each configured input target for its stream_object property and appends successful values. Run26 observed no transferred values. This means the generic collected-stream-object transfer was not the path supplying this ADM DDP invocation. The observation does not distinguish an empty input-target list from missing stream_object properties on targets.

## Keep run24 and run26 separate

Run24 separately captured DDP route tables in one route-consumer call: 38 frame channels, ten numeric category routes to slots 0..9, and 28 additional record indices to slots 10..37. Run26 did not capture route-table values; it only observed the empty collected input list and same-thread DDP reachability. Do not join run24 route slots to run26 input properties by ordinal or job similarity.

## Correction and remaining identity gap

The prior claim that AtmosMezz Run directly wrote stream_object is withdrawn. Its static body does not show that write, and the candidate virtual producer was not hit. The ADM file-path reader/decoder and PCM pointer-table producer remain unknown. DDP frame slots remain unbound to CHNA UID, physical track, global source ID, and selector columns. No current-product output, full-song encode, U04 numeric run, Gate1, or playback acceptance is claimed.
