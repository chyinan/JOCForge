# F06 selected DPLM K-filter exponent cold initialization

Native static evidence closes one field-level fact: for the selected six-channel 48 kHz DPLM path, the numeric-state constructor initializes the per-channel K-filter exponent as signed I32 zero. The separate energy-accumulator exponent is signed I32 with cold value 48.

The constructor-created numeric state flows through the per-channel row object to the selected DPLM callback and reducer. The reducer reads the K exponent while normalizing filtered power; the scoped selected DPLM processing chain contains no K-exponent write. It separately updates/resets the energy exponent to 48 when a complete window is consumed. Fresh worker-root setup routes reach the same numeric-state constructor. No separate in-place K-exponent reset was observed.

This supersedes any use of synthetic V4 K exponent [48;6] as native cold evidence. Keep that recipe synthetic. The native inner-state object is not yet joined by runtime owner/generation receipt to canonical owners.meter, so the production canonical root and terminal receipt remain unbound. No native execution or numeric replay was run.
