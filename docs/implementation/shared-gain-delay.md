# Shared gain delay

Status: `LOCALLY_VERIFIED_SELECTED_STATE_AND_CALLBACK_TRANSITION__METER_PARENT_DEFERRED`.

The selected meter-side gain-delay leaf owns one callback counter, the eight-field parameter state, and six 32-sample histories. `SharedGainDelayState::canonical_bytes` serializes the complete 812-byte `SDVSTATE` portable value. `reset` reconstructs the cold value; neither staging nor failure mutates the committed input state.

`stage_shared_gain_delay_callback` consumes one six-row, 1,536-sample callback and executes 48 ordered 32-sample blocks. Each block applies the shared peak and gain transition before the six delayed filter rows. The returned `SharedGainDelayTransition` owns the complete after-state and output words, but has no owner installation or commit authority.

The selected parameter constructor admits only the approved `receiver_blend32` Q15 record. Arithmetic tests cover signed zero, attenuation, counter overflow, malformed input, and cross-callback history. A compact local fixture preserves the six approved continuous input recipes and post-computation output/state digests; all six cases pass from their declared reset/continuation state.

This slice does not implement the independent meter frontend, OAR input producer, pending 1,024-sample lifecycle queue, K-window composition, whole meter root, finalize transaction, or real-programme validation.
