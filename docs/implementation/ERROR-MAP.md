# 错误 outcome 映射

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

| functional_id | error_code | category | trigger | programme_state_after | caller_action |
|---|---|---|---|---|---|
| XSTJ | `CLOCK_CONVERSION_RANGE` | INPUT_REJECTION | zero sample count or checked clock range failure | UNCHANGED | reject before state or output creation |
| F02 | `ZERO_SAMPLE_COUNT` | INPUT_REJECTION | zero sample count or checked clock range failure | UNCHANGED | reject before state or output creation |
