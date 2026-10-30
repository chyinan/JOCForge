# ID14 semantic parser and complexity evidence

## Model fact

The frozen R33 model has a semantic EMDF/ID14 parser. It requires exact container consumption, validates the selected OAMD/ID14 profile, parses the JOC header, and derives `U8(object_count + 1)` for the selected model path. The selected profile uses 15 JOC objects and returns model value 16. The frozen-model probe contains 13 matching debug/release observations. A successful complete selected OAMD-plus-ID14 wrapper execution remains `NOT_RUN`.

This is a model rule. It does not establish that DEE computes the same value from ID14 object count.

## Bounded native fact

Static tracing found an ID14 writer/queue path and a generic payload collector/parser path. In the examined selected path, the generic parser copies payload structure; no ID14-specific semantic decoder call was observed. Native valid or malformed ID14 parser execution was `NOT_RUN`. This bounded no-hit is not a global parser-absence claim.

The selected native BSI working-header byte came directly from a configuration record. Prior native captures tested configuration parameter 13 values 1 and 16; those summaries were rehashed, not re-executed for this package. The native call order places ordinary codec/BSI work before the outgoing EMDF serializer.

The model's `object_count + 1` result and the native configuration byte are distinct producers. Their selected numeric value may both be 16, but no owner, generation, or same-input typed join connects them. Native derivation from ID14 object count remains `NOT_OBSERVED`; the native count-to-configuration-byte join remains `UNBOUND`.

## Boundaries

The six accompanying vectors contain inputs only and assert no outcomes. Complete selected wrapper execution, native semantic parser execution, and whole F13C readiness remain `NOT_RUN` or false. This supplement contains no native addresses, decompiled source, binary, PCM, encoded output, or private key.
