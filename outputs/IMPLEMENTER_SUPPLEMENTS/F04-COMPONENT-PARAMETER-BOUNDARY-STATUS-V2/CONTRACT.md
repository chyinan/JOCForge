# F04P / F04PG component-parameter boundary status

Classification: HASH_BOUND_STATUS_CROSSWALK. The selected F04P/F04PG model formulas, ranges, cold state, recurrence and callback proposal boundary are already in the approved package. This supplement records their status and the remaining native identity/storage limits; it does not add an algorithm rule.

## Closed model rules

The selected profile uses the approved fixed 48 kHz / 64-sample parameter generator, eight Q15 parameters and one shared parameter state over 15 component rows. The approved F04PG/F04P specifications and provider references listed in SOURCE-CHAIN.json remain the source of the exact formulas, parameter words, binary32 behavior, cold state, recurrence, range checks and reset rules.

The frozen Python source identity was independently rehashed at 6,733 bytes. It declares hold and blocks_processed as Python int; hold is bounded to 0..=0x7fffffff, while blocks_processed is nonnegative and increments once per 64-sample transition without a model upper bound. Its constructor accepts only the scalar tuple sample_rate=48000, row_count=15, block_count=64 and contains no admitted-configuration object or install-call token. Python integer behavior does not establish a native fixed-width type.

## Corrected native-target status and remaining gaps

The target module identity is now independently rehashed: dee_audio_filter_ddp_atmos.dll, 10,942,464 bytes, SHA-256 recorded in SOURCE-CHAIN.json. The earlier status that no target image existed in the relocated research root is superseded. Static analysis has still NOT_RUN because no usable IDA/Ghidra interface or nonempty database was available to Wash/R4.

The native storage owner/type for hold remains UNBOUND. The native storage owner/type and overflow behavior for blocks_processed remain UNBOUND. The admitted profile instance to parameter-install owner/call identity remains UNBOUND. Do not infer native storage from Python int or choose a native owner from field names. No native disassembly or execution is claimed.

The vectors are input-only with no expected outputs. The approved model formulas remain usable independently; this package does not establish native storage parity, whole F04 receiver execution, real ADM output or product acceptance.
