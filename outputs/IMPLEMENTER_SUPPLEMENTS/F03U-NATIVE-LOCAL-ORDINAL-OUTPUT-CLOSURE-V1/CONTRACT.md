# F03U local-ordinal source-weight/output boundary

## Wash decision for the selected in-DLL encoding path

The selected native DLL path materializes per-source feature buffers by local ordinal, selects/reassembles matrix rows using local-ordinal arrays, passes matrix views and dimensions to scoring/source-weight consumers, then serializes per-source F32-array pointers by ordinal into the host-facing byte buffer. No typed global candidate ID is read by the traced materializer, passed to the source-weight/score calls, or serialized in the per-source F32 metadata payload.

For this bounded source-weight-to-output-byte path, the clean model should preserve the native local-ordinal/matrix-column order. A global candidate ID input is not required by the traced source-weight API and should not be added solely to satisfy the earlier “global ID → matrix column → weight” question. The prior UNBOUND label is superseded only for this in-DLL execution path: the evidence supports that the traced path does not consume a global ID.

## Boundary retained

The host or ADM graph consumer after the DLL returns its bytes is outside the target DLL and was not traced. Any post-DLL graph-ID to exported-object join remains unbound as a separate integration question. Run 120 captured matrix/weight hashes but not candidate identities or which materializer branch ran; run 121 did not reach the native callback. The frozen numeric model also has no ID-to-column API. This decision does not claim cross-system ID identity, whole-chain readiness, model/native output comparison, or audio acceptance.
