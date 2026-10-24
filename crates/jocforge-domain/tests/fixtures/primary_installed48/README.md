# Primary installed 48-source activity and membership supplement

This Wash-reviewed package is for internal implementation of the selected ADM-to-JOC profile only. It adds a complete selected-source mapping/activity producer contract and the selected 11-point membership contract to the already approved `IMPLEMENTER_PACKAGE`. It is not a product-completion or native-acceptance certificate.

Implement from `ACTIVITY-AND-ROUTING-RULES.json`, `MEMBERSHIP-SPECIFICATION.md`, and the approved constants and input vectors. `INSTALLED48-SOURCE-MAP.json`, `CONTROL38-DEFINED-RANGES.json`, `CALLBACK-ACTIVITY-OUTPUTS.jsonl`, `FLAG-TRANSITIONS.json`, and `POSTROUTE-VARIANTS.json` are expected outputs for post-computation comparison. Compute from the authorized `test-vectors/full-song/input-adm.wav` input before opening these expected-output files; never use expected rows as producer seeds.

The selected profile is 38-channel signed PCM24 at 48 kHz, processed in 1,536-sample callbacks. It maps 11 virtual prefix rows followed by 37 regular physical-host rows, excluding the selected direct-LFE host and sorting remaining hosts physically. Owner identity is physical host plus the original semantic UID/track/stream/channel/pack/object strings; no arithmetic key is derived from an object identifier. The package records only defined control ranges and explicitly marks unconsumed ranges unknown.

The activity contract covers the complete primary run: 4,247 normal callbacks, four cached-frame drain callbacks, 4,251 total callbacks, 48 source rows, 5,695 field transitions, and 71 observed post-route variants. The final normal callback consumes 1,050 source samples and pads 486 zeros. Drain callbacks reuse the last full padded frame and held controls; they do not consume more ADM samples or advance ADM event time. Follow the F32 window operations, strict `> 2^-23` witness rule, history updates, writer ordering, field0/field4 semantics, and reset behavior exactly as specified.

The nine spread/gain cases intentionally use their declared synthetic basis. They validate the complete F04G arithmetic on those inputs; they do not represent primary-source native gain11 observations. The selected P11 basis is a separate approved 33-word vector. Primary membership is model-derived and has not been independently validated against native gain outputs. Implement the full specified algorithm; do not omit it because that native comparison is pending.

Validation evidence is in `WASH-VALIDATION-RECEIPT.json`. The full PCM payload is not included. The authorized WAV may be read in memory for tests; do not persist or hash PCM samples. Full identity of the separate primary WAV remains `NOT_REHASHED`, and native encode, listening acceptance, and product acceptance remain `NOT_RUN`. The private vendor key is excluded.

Internal implementation use only. Do not publish or distribute this package to third parties.
