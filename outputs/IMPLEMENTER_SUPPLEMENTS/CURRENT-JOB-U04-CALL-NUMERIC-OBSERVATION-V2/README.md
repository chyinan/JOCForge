# Native U04 helper call evidence

This package binds the captured helper call to the exact PreparedJobInput XML, canonical normalized-config identity, and authorized ADM-BWF input hash. The v9 input and output slices come from the same helper call. A v8 same-config replay independently produced identical output bytes. Neither observation run completed the whole encode; a separate v4 run completed the reference DEE EC-3 output, whose payload is not included.

The second observed helper call uses raw mode 0 and range [0,7), and returns raw value 0. Its output is a 7x20 F32 buffer; the exact 20x7 consumed input slice uses row stride34. The 140 F32 input/output products reproduce the native output when each spatial row is multiplied by its inferred F32 scalar. This is a helper-level numeric observation, not a model replay or a proven source-weight vector.

Global candidate IDs, the join from this local input slice to the approved band-major 24xN F03U source-feature snapshot, source callback generation, downstream consumer, Type10, and six-plane composition remain unbound or not run.

The first observed U04 entry returned zero, but its output snapshot was not captured. The included numeric vector is for the second call.