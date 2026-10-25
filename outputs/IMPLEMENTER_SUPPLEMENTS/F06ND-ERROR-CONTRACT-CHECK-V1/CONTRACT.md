# F06ND error-contract scope

This is a clean-spec audit of F06ND.7 and its existing washed clarifications. It does not infer native error parity or select new local error policy.

## Declared F06ND preflight and outcomes

The approved preflight order is:

1. required parameter supply and identity;
2. profile/configuration domain;
3. state type and dimensions;
4. source-generation agreement;
5. primary/secondary same-generation shape and finite checks;
6. programme dialnorm domain;
7. computation.

The source explicitly maps missing parameters to `PARAMETER_UNAVAILABLE`, unsupported profile/configuration domain to `UNSUPPORTED_PROFILE`, state/type/shape/generation errors to `DRC_INPUT_ERROR`, and intermediate non-finite arithmetic or overflow to `NUMERIC_FAILURE`. It lists dialnorm domain as a separate preflight gate, then uses the generic phrase “out of domain” for `UNSUPPORTED_PROFILE` without explicitly tying that mapping to dialnorm; keep the out-of-range dialnorm code `UNBOUND` rather than broadening the profile mapping. Any declared failure returns no normal output, commits no partial numeric state, retires the lifecycle, and requires full cold reset.

The clean source-generation clarification also classifies checked U64 scheduler generation overflow as `NUMERIC_FAILURE`; this is a clean implementation contract and makes no claim about native wrap parity.

## Exact edges that remain unspecified

- The materials list the gate order but do not state a tested first-error result when multiple gates are invalid simultaneously. Do not label that ordering as a certified simultaneous-invalid precedence.
- Raw input NaN/Inf found during the finite preflight is not explicitly assigned to `DRC_INPUT_ERROR` or `NUMERIC_FAILURE`.
- A supplied parameter identity mismatch has no defined distinct outcome.
- F06ND.7 lists a dialnorm-domain gate, but does not separately map an out-of-range dialnorm value; keep that classification `UNBOUND`.
- The approved materials do not map each nested statistics/scheduler/normalizer/quantizer helper's internal error label to a wrapper code. Only the F06ND-level class “intermediate non-finite/overflow” is explicitly `NUMERIC_FAILURE`; do not infer passthrough or translation of other child labels.

Existing clean vectors cover valid statistics/scheduler/quantizer inputs and whole-composition lifecycle cases, including one downstream failure/retirement path. They do not provide simultaneous-invalid first-error vectors or the edge classifications above. Treat those as `UNBOUND`; do not invent outcomes or claim native parity.
