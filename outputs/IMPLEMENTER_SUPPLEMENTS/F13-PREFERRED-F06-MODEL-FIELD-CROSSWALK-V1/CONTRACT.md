# Final measurement kind to ID1 preferred

Scope ready: the frozen duration-model producer/caller connection. Native measurement-member identity and selected native factory/reset lineage remain open.

## Typed model connection

The actual duration timeline tool reads `Root44_gating_kind24` from its dialogue measurement-result document. The value must have exact integer type (a boolean is rejected) and belong to {0,1,2}. It computes preferred as `(public_measurement_kind == 1)`. The timer configuration stores that boolean as canonical I32 0/1 at ordinal 2. This is an existing internal measurement result, not a user task input.

The domain matters. Approved F06.2 defines internal kinds ungated/level/dialogue as 0/1/2 and public result kinds as 0/2/1. Thus public result kind 1 denotes the dialogue branch; internal kind 1 denotes the level/ITU branch. Do not substitute the integer from one domain for another. Use a semantic type or an explicit checked conversion at this boundary.

The measurement producer retains its selected-result kind in its semantic result dictionary; the duration caller consumes that field from the supplied document. This establishes a producer-to-consumer model connection. It does not establish that the separately recovered native 44-byte measurement member is this same field merely because both can contain 0/1/2.

## Selection authority and precision

Preserve the current approved F06.2, F06.2b and F06U selection rules, finite F32 precision, enum conversions, and independent whole-programme measurement requirement. In particular, unavailable ITU must use the freshly computed F06U ungated fallback. The recovered historical scalar helper admits a narrower valid-ITU subset; its rejection outside that subset must not remove the approved ungated path.

For the selected DI-disabled model configuration, a valid ITU branch produces internal kind 1/public kind 2, so model preferred is false. The approved ungated fallback produces public kind 0. These are consequences of model rules, not observations of the selected native provider or permission to hard-code false before executing measurement.

## Caller and execution scope

The older 96-call tool with a literal preferred=True is a different exact source. The inspected duration-stage ledger names the duration caller that reads the typed result kind. The normal duration assembler consumes timeline-produced ID1; its own freshly constructed configuration is only used by its fallback branch. Do not identify that fallback as the admitted normal ID1 source.

The inspected stage ledgers are candidate-qualified historical execution records, not a current model execution or proof of DEE whole-programme byte parity. No such parity is required for this interface rule.

## Implementation use

An F06 model-owned final public result kind can now feed the checked ID1 preferred conversion on the model path. Keep measurement/config/programme identity attached through the implementation's existing ownership contract. Do not require a DEE-native object with the same Rust receipt name or invent a new user input. Native member-to-public-kind identity, selected native branch/value and runtime reset lifetime must remain separately qualified until their actual dataflow is delivered.
