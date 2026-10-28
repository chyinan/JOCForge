# Meter-root exponent field crosswalk

The existing V1 field window_k_energy.exponents_i32 is the F06W.5 energy-accumulator exponent array: signed I32[6], cold 48. It is not the K-filter exponent.

The native selected DPLM constructor provides a distinct signed I32[6] K-filter exponent with cold zero; the scoped DPLM chain reads it and has no K-exponent write. The local canonical root should serialize separate named arrays if it represents both native state members. Keep K-filter F32 history and energy mantissa at positive zero.

This is a local canonical-schema crosswalk over approved leaves plus a washed native static fact, not a native root ABI claim. The DPLM-to-GTC owners.meter mapping, canonical digest, MeasurementReceipt and parent commit remain NOT_RUN/LOCAL_INTEGRATION_PENDING. The old [48;6] K value is synthetic only.
