# Selected input, flush and EOF lifecycle addendum

This addendum covers one bounded selected run with four external process calls and five flush calls. Each process call carries 38 source-index/control-position records. The API control positions are 0, 1536, 3072 and 4608; they are not proven absolute offsets into an ADM/WAV file. The PCM pointer layout is observed only as tokens/offsets and is not sampled or hashed.

Frontend source calls 1 and 2 occur inside external process calls 3 and 4. External calls 1 and 2 are renderer warmup and do not call frontend. Frontend source calls 3 through 6 occur in active flush calls 1 through 4; each produces a caller-visible 4096-byte AU. A fifth flush call is empty, returns 0 and produces no frontend/source or planner advance.

After EOF, gate progress remains 4608, planner counter 6, first block 2, flush-started 1 and flush-remaining 0; one pending ID14 descriptor/body of 218 bytes remains in the old workspace. A process call after flush has started returns raw U32 4294967293 / signed I32 -3, writes output length 0 and consumed flag 1, and leaves flush-started/remaining unchanged. Empty flush with flush-started 0 or 1 returns 0 and length 0 while preserving that flag.

Close returns 0 and removes the JOC runtime without clearing the observed pending descriptor, gate or planner scalar fields. The descriptor and body are inside the workspace allocation which is then freed; no read occurs after free. This is owner retirement, not semantic queue-clear and not a same-owner reset. One cold initialization and one close/free lifetime are observed; same-owner reset/recreate remains `NOT_RUN`.

Renderer/backend gate progress saturates at 4608. It is a warmup accumulator, not an advancing source clock. Source control positions, renderer delayed sample positions, absolute file offsets and host clock must remain separate until an additional native join is delivered. The diagnostic producer length inputs are boundary inputs, not model-generated protected JOC body lengths.
