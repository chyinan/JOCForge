# F06W output-sample accounting ledger

This internal-only slice covers F06W.1/.2 integer sample accounting and
callback spans for an explicit positive source sample count. It processes no
audio samples and owns no persistent meter state or programme clock.

For each source length `N`, normal callbacks are `ceil(N/1536)` and the final
normal callback includes positive-zero padding. The fixed latency is 3104
samples, followed by three zero-input drain callbacks. The ledger maintains
startup, pending, and processed sample counts; each published span records its
callback index/phase, raw-frame begin, number of valid samples, and
programme-sample begin. It requires startup and pending to reach zero and
processed to equal `normal_callbacks * 1536`.

The output also counts the complete 1024-sample PCM prefix, the complete 960
windows in that prefix, and the separate PCM/energy remainders. These are
integer accounting results only: this code does not call K-filter, power_sum,
F06L accumulator/window/reduction, create six-lane window state, or emit an
F06I energy record.

The five synthetic `schedule_cases` are checked by input length and complete
span/counter projections. The same fixture contains ten F06I cases and five
F06W.3/.4/.5 dependencies elsewhere; those are not consumed here. No real
audio, ADM, historical meter input, or product state is used.

Source and delivery identities are in
`meter-output-schedule-provenance.json`. The slice is internal-only and must
not be committed or distributed through the public branch.
