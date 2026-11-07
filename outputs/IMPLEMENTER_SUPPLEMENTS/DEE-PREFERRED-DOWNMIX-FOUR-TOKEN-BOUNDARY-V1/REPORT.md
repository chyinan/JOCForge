# Preferred downmix mode: four-token boundary

Reference-only Wash supplement; it is not an implementation specification. `IMPLEMENTATION_READY=NO`. No token is assigned a rendered sound or metadata effect.

The sealed source receipt binds the DDP XML parser's static codes: `loro→0`, `ltrt→1`, `ltrt-pl2→2`, `not_indicated→3`. It does not bind these codes to the value stored at `MixMode+0x50`: the typed setter writes a helper's readback of the pre-existing field because its optional callback is null. Effective omitted/empty XML materialization is not captured.

The first typed reader of the stored field is `FUN_180171f50`; the only direct caller in the bounded receipt is formatter `FUN_180171ce0`. A call from the selected encode pass to that formatter was not established, and the scoped `FUN_1801475f0` body has no direct `Priv+0x980` read. The selected processing consumer and audio/BSI effect remain `UNBOUND`.

DME is separate: its UI maps `Stereo→loro`, `Surround→ltrt`, and `Not Indicated→not_indicated`; there is no DME UI value for `ltrt-pl2`. The DME job model produces JSON and is not joined to the requested DDP XML field. Do not turn this DME mapping into a DEE XML writer.

No native source, decompilation, media, PCM, encoded payload, key, or license is included.