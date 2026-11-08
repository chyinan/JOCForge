# Metering-mode token and runtime boundary

The static map assigns raw U32 candidates 0, 1, 2, 3, and 4 to 1770-4, 1770-3, 1770-2, 1770-1, and LeqA. These are candidate values, not five established algorithms.

A generic string-map consumer conditionally writes a four-byte value at record +0x90. The scoped downstream validator does not read that offset; the selected provider and first effective measurement consumer remain UNBOUND.

The one new 1770-3 arm naturally exited with code 10, no scalar, and no output. Its cause is UNBOUND and it was not retried. Prior 1770-4/LeqA observations show sample-dependent measurement diagnostics; they are reused, not new runs, and the measurement unit was not stated in the native log.

Do not infer that 1770-3 is invalid, that all five tokens are supported, or that a unique algorithm exists per raw value. Audio/JOC effect is not inferred.
