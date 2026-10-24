# Clip and coefficient generation connection

Functional Core connection contract; production composition remains pending.

The washed supplement `F13C-CLIP-COEFFICIENT-GENERATION-JOIN-V1` has manifest SHA-256 `24a0c4d21df35aa36450ec405f80cc836a52228f697ff509f16d0dead997cb91`. Its distributed manifest binds CONTRACT.md and VALIDATION.json. The coordinator reports eight upstream observation payload bindings and ordinary/optimized causal-checker passes; those upstream checks are distinct from the two distributed payloads.

| Boundary | Required immutable values | Existing implementation |
|---|---|---|
| Clip to coefficient production | One parent generation, representation context, clip index and approved sequence | `clip_producer.rs` supplies the explicit sample-boundary clip result; coefficient/fit producer composition remains open |
| Coefficient to quantization/body | Same representation descriptors, selected channel table and coefficient rows | `coefficient_payload.rs` supplies controlled quantization and Huffman/body writing |
| Body to payload writer | Same generation header, coefficient scratch and valid-bit count; generation-bound clip and sequence | Controlled writer exists; parent generation admission remains open |
| Writer to ID14 queue | Produced bytes and returned length as one result; separately owned queue copy | Controlled writer exposes a successful owned descriptor copy; production metadata-owner admission remains open |

The observed secondary natural generation joins source invocation 3, frontend invocation 1, clip generation 0, coefficient point 0 and sequence 0. These observations do not define a clock conversion. Equal ordinals must not be used to join producer results. Existing approved clock contracts govern production indexing.

Observed clip 128, 900 coefficient bits, writer capacity 372 and returned body length 132 are observations only. They are not constants, fixture input bytes or evidence that coefficients are zero. No coefficient values or payload bytes were captured by this supplement. Distinct queue storage establishes ownership separation without independently proving content equality.

The earlier claim that the frozen model lacks peak/hold and obtains clip only from descriptor target gain is withdrawn by `F13C-CLIP-SOURCE-CORRECTION-V1`. Both approved algorithm material and generation-connection evidence must be retained when assessing the producer. Explicit clip input remains necessary until parent composition is admitted.

Required before production connection: approved parent cold construction and reset projection; representation-context identity across clip/fit/writer; conversion from parent generation to existing typed clocks; retry and failed-stage behavior; authority to admit the resulting ID14 body to the metadata owner. This supplement supplies no replacement formulas for those rules.

The subsequent F13C-CLIP-INTEGRATION-REFERENCES-V1 (manifest `45f9c70dcb0c6c305b83960a9f47984b18181f304add08fc2b27e582dc654f17`) locates these existing approved rules. They are implementation/integration obligations, not broadly missing specifications: spatial_audio alone advances descriptor and temporal3; J05 preserves that execution; source s is the writer generation, dispatch is s+1, and reset reconstructs the whole attempt. The receiver proposal now retains selected index, target gain and the actual s09_return buffers rather than discarding them. These fields do not create another clip owner. Completion of the consuming J05/parent join still requires implementing its existing lineage admission.

SELECTED-INPUT-APPLICATION-REFERENCES-V1 (manifest `f81902555ecacdffaf1f9af5e36c09f9c37be16fd2eb9339c88e870b8760ae54`) authorizes the local application value boundary. `prepare_selected_job_input` normalizes XML using the adapter's validated descriptor/sample extent and retains actual immutable resource bytes and SHA-256. Its tests cover three accepted/eight rejected configurations and immutable resource binding. The adapter's same-resource RIFF validation, metering, programme construction and production caller are separate remaining integration work. The prepared value alone cannot authorize READY or encode success.

`CoefficientPayloadOutcome::into_metadata_body` now consumes the local writer result and transfers its successful descriptor copy into `MetadataPayloadBody` with an explicitly supplied `CodecSourceIndex`. It preserves the produced bytes and length without another serialization. Writer failures produce no metadata body. Tests exercise successful and failed controlled cases. This value adapter does not stage a metadata owner, prove generation admission or introduce a clock conversion.

Primary and later-generation coefficient comparison, audio comparison, complete cold-owner construction and end-to-end acceptance remain NOT_RUN or NOT_READY. Causal observation checking is not DSP execution.
