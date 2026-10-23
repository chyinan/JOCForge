# F06U single-slot ungated mean

This internal-only slice implements the selected F06U period-one/capacity-one
mean state transition and its fixed 1770-4 dialogue-intelligence-off getter.
It is not an integrated meter or production measurement pipeline.

## Mean state and admission

The state is the complete eight-field 40-byte `MEAN_STATE_V1` value:
`count`, `mean`, `exponent`, `phase`, `cursor`, `filled`, `linear_energy`,
and `result`, serialized little-endian after the ASCII header `UMSTATE1`.
Cold/reset is count 0, mean +0, exponent 30, phase/cursor/filled 0,
linear_energy +0, and result -15/16. Only a finite nonnegative mantissa with
integer exponent 0..=30 and the enabled selected writer configuration is
admitted.

For each explicit `(mantissa, exponent)` pair, the F06L energy metric is
compared with R32(-70/128). Equality calls the writer; below-threshold or
zero-activity inputs preserve all eight old fields and do not create an event.
Each admitted input is one period-one event. The implementation preserves the
separate F32 operations, the signed U32-to-I32 count conversion, post-result
count halving at the 2^31 boundary, one-pass 1/2 normalization, and the
specified eight-field state projection.

The F06L log/energy network and its exact hash-bound constants are reused from
the internal-only log provider. The F06U path does not load the true-peak FIR
parameters or execute K-weighting, windows, the integrated meter, or audio.

## DI-off getter

The selected getter accepts only `1770-4`, dialogue intelligence false, and
F32 speech threshold exactly 100. It returns dialogue -120 and speech
percentage -1 sentinels without running a speech classifier. A supplied ITU
internal result strictly above -15/16 is selected with internal/public gating
kinds 1/2. Otherwise the actual F06U result is selected with kinds 0/0. If
the ITU measurement is unavailable, only the F06U result is used; no legacy
helper or song-derived value is used as a substitute.

## Evidence and limits

The implementation is checked against ten washed mean sequences (102 event
steps), four coupled state projections, four entry rejections, four valid
DI-off getter cases, three unsupported getter configurations, and the legacy
`ITU_host=-120` observation resolved using the actual F06U cold result. In the
coupled case with available ITU, only the F06U terminal state is checked here;
F06I and its selection branch are not implemented.

The 6,794-entry historical energy files are explicitly excluded from local
fixtures, implementation inputs, and runtime data. They remain separate
historical acceptance artifacts only. No real ADM, meter frontend, F06I,
F06W, K/FIR, whole none-DRC composition, measurement owner, parent transaction,
W04, or product acceptance is claimed.

This slice and its constants are internal-only. Do not commit them to or
distribute them through the public branch.
