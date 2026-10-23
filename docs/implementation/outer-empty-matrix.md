# F03S.1 empty-target matrix skip

This direct slice handles only the `n=0` outer matrix case. It validates the explicit source-call order as a unique subset of `0..48`, returns an empty matrix with shape `[0,48]`, records zero solver calls, and marks empty targets skipped. The API has no response or XYZ inputs, so the empty branch cannot accidentally read or synthesize them.

Nonempty target sets remain the responsibility of a different outer path; this function rejects `n>0` rather than padding to capacity 15. It owns no state or clock and does not call NNLS, F03S energy/correction, selector, U04, or a parent transaction.
