# 错误 outcome 映射

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

| functional_id | error_code | category | trigger | programme_state_after | caller_action |
|---|---|---|---|---|---|
| XSTJ | `CLOCK_CONVERSION_RANGE` | INPUT_REJECTION | zero sample count or checked clock range failure | UNCHANGED | reject before state or output creation |
| F02 | `ZERO_SAMPLE_COUNT` | INPUT_REJECTION | zero sample count or checked clock range failure | UNCHANGED | reject before state or output creation |
| F04D | `RECEIVER_PARAMETER_UNAVAILABLE` | provider availability | required selected threshold, descriptor scalar, or PG03 gain provider is unavailable | RETIRED | No index, ramp output, after state, or J05 receipt; I03 STAGE_FATAL retires the programme; committed descriptor state stays unchanged; do not retry the same attempt. |
| F04D | `RECEIVER_PARAMETER_INVALID` | provider identity/shape/finite/gainpositive | provider identity, shape, finite-word, or positive-gain validation fails | RETIRED | No index, ramp output, after state, or J05 receipt; I03 STAGE_FATAL retires the programme; committed descriptor state stays unchanged; do not retry the same attempt. |
| F04D | `F04D_HOLD_RANGE` | state shape/range | hold semantic U32 cannot be decoded from canonical hold_count in 0..0xffffffff | RETIRED | No index, ramp output, after state, or J05 receipt; I03 STAGE_FATAL retires the programme; committed descriptor state stays unchanged; do not retry the same attempt. |
| F04D | `F04D_INPUT_SHAPE` | input shape | control or post-temporal3 ramp data is not six lanes x 1536 samples | RETIRED | No index, ramp output, after state, or J05 receipt; I03 STAGE_FATAL retires the programme; committed descriptor state stays unchanged; do not retry the same attempt. |
| F04D | `F04D_INPUT_NONFINITE` | input finite | a control or post-temporal3 ramp sample is nonfinite | RETIRED | No index, ramp output, after state, or J05 receipt; I03 STAGE_FATAL retires the programme; committed descriptor state stays unchanged; do not retry the same attempt. |
| F04D | `NONFINITE_INTERMEDIATE` | numeric intermediate | a control or ramp intermediate/output becomes nonfinite after valid providers, state, and finite inputs | RETIRED | No index, ramp output, after state, or J05 receipt; I03 STAGE_FATAL retires the programme; committed descriptor state stays unchanged; do not retry the same attempt. |
