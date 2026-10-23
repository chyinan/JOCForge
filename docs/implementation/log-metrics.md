# F06L direct log and peak metrics

This internal-only slice verifies the existing F06L ordered-log,
energy-metric, sample-peak, and true-peak metric functions, the F06L.7
normalized-energy accumulator, the F06L.8 window-end normalization, and the
F06L.9 ordered reduction against their washed scalar vectors. It adds no
parallel log/peak network.

The selected hash-bound log constants feed the existing separately rounded
F32 polynomial network. Direct assertions cover 14 log cases, 8 normalized
energy-metric cases, 6 sample-peak cases, and 8 true-peak metric cases. Each
sample-peak recipe starts from 1024 positive-zero words, writes the declared
negative peak and equal positive peak, then verifies peak magnitude,
first-maximum tie behavior, internal metric, and host metric.

The direct tests cover 7 accumulator, 6 window-normalization, and 7 ordered
reduction cases, plus the three washed window-entry rejections. Contract-derived
checks cover invalid accumulator and reduction domains, nonfinite window
mantissa, and sample counts outside 1..=48000. The separate F06TP FIR/cascade
tests and their state are not replaced or rerun by this metric test.

The source package and local-review identities are recorded in
`log-metrics-provenance.json`. The review status reports only its own local
scope; this result does not claim complete F06, a full meter, real ADM, or
product acceptance. Parameter distribution remains internal-only.
