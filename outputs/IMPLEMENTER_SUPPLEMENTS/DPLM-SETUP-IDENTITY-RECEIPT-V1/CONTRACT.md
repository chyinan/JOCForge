# DPLM setup identity receipt

Hash-bound static evidence closes the native DPLM owner/object chain from setup to the selected K-filter reducer. The DPLM instance stores a worker-root member at setup and loads the same member in the selected 48 kHz process wrapper. The worker root owns a channel row table; each row owns a numeric meter-state object; the per-row callback passes that same numeric-state member to the reducer.

The numeric-state constructor cold-initializes the selected six-channel K exponent as signed I32 zeros and the separate energy exponent as signed I32 48. The reducer reads the K exponent and updates the energy exponent. This is static object flow and constructor evidence, not a runtime pointer/generation capture.

The native DPLM worker root is not an ABI-equivalent GTC owners.meter token. Mapping it into the local GTC meter root is a clean-room parent integration decision: mint a local owner/generation receipt there; do not copy a native pointer or claim runtime identity. Native execution, terminal meter digest and MeasurementReceipt remain NOT_RUN.
