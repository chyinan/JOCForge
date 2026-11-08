# DME downmix property to native XML writer boundary

The sealed static receipts bind DME host values and active flags into a composite --encoder child argument, and bind clientJob state to JSON. They do not bind that composite to a native DDP XML writer or leaf. Host property keys differ from the native XML keys.

The native parser path separately maps preferred_downmix_mode tokens to a 32-bit ordinal and calls the MixMode setter. Its optional callback is null, so the helper returns the pre-existing MixMode+0x50 value; ordinal-to-effective stored value remains UNBOUND. The separate Priv+0x210 StringType output member is not joined to the mode or level values. Four level parser results remain helper-result uint slots with unbound dB/BSI/typed-descriptor meaning.

No DEE/A-B run occurred. No 2ch downmix, Atmos/JOC audio, BSI, or level effect is established. The first unbound edge is DME composite argv to native provider/XML-writer input.
