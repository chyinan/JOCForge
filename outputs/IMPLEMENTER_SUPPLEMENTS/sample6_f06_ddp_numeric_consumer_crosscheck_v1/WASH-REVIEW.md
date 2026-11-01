# Sample-6 F06 and DDP numeric-consumer cross-check

This is a washed, sample-specific evidence supplement. Its source intake package manifest and seal hashes are recorded in `VERIFICATION.json`; the intake package's eight sealed files and all thirteen referenced static evidence hashes were independently checked. The intake checker passed under normal Python and `python -O`.

## Observed in one completed DEE job

- A Float32 F06 result was copied into a cache field on the same DDP owner, then copied into an encode-parameter record. The same bits, `0xc1864780`, were observed at each step. The generation token remains unbound, and the value's unit is not stated.
- `custom_dialnorm` was absent from the job XML; its getter returned zero and the conditional overwrite did not execute. This run therefore does not reveal the nonzero override conversion.
- A DDP row was copied into a distinct internal row buffer, then one Float32 lane from that buffer was consumed by a four-lane addition in the same call chain. The observed lane was `0x38158000`. The other lanes were not inspected.

## Still open

The run did not hit the selected PCM24 reader conversion/store, the writer or staging path for the DDP owner's DialNorm U16, the five-bit serializer, or the selector-3 setup callback. No input PCM sample was joined to the DDP source row, and no owner/generation join was captured. Do not infer a PCM conversion result, DialNorm formula, or setup/CHNA mapping from the observed scalar values.

## Output observation

The one-job output was hashed before a metadata-only OpenJOC 0.18.0 inspection. It had 4,247 parsed access units with JOC payloads, profile index 3, 15 coded rows/objects, and complexity index 16. Deployed-compatibility validation passed; strict validation failed for all 4,247 access units. The output file was removed after inspection, so this is not a reusable artifact. Rendering and playback were not run. This is not final output acceptance.

## Scope for implementation

Evidence-only. It may guide investigation and integration planning for the specific observed copy/consumer boundaries. It does not approve audio-algorithm behavior, infer a measured-loudness-to-DialNorm conversion, or close the end-to-end ADM reader-to-encoded-row lineage. Keep `implementation_ready=NO`.
