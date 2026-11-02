# Atmos DDP plugin-local DRC property registry boundary

## Wash disposition

The upstream package was independently checked against its manifest, seal, SHA256SUMS, and payload hashes. Ghidra 12.2 evidence closes only the plugin-local property registration and qualified-alias lookup boundary described here. The package status is `NOT_RUN`; no effective host XML value or runtime setter behavior was observed.

## Static plugin-local facts

- `FilterDDPJocEncPlugin` vtable slot 1 (`+0x8`) dispatches the generic property setter. Ordinary keys are resolved through the plugin's property registry; the returned descriptor's `+0x28` setter receives the value string.
- The plugin schema registers `line_mode_drc_profile` and `rf_mode_drc_profile` as `xs:string` properties with enum constraints.
- The registry owner contains the corresponding descriptors at `owner+0x6d0` and `owner+0x750`; the registered aliases are `drc:line_mode_drc_profile` and `drc:rf_mode_drc_profile`.
- The plugin-local resolver compares the complete qualified alias. Whether the external host prefixes the XML leaf key is not established by this package.
- The static unmatched-key and multiple-match branches enter the `ParametersContainer` error path and call `RaiseException`.

## Not established

This does not identify enum ordinal/value mapping, XML defaults, host normalization, StageRunner handoff, effective values for a selected job, the descriptor setter's value transformation, or any DRC/audio DSP behavior. Do not implement an audio DRC rule from these registry facts alone. No DEE run, PCM, payload, key, or license material was accessed.
