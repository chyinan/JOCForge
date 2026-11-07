# Lo/Ro center mix-level owner boundary

Reference-only Wash supplement. It binds a static typed member, not a selected runtime object or BSI consumer. `IMPLEMENTATION_READY=NO` for audio effects.

The XML key `downmix:loro_center_mix_level` is statically associated with a separate `MixLevel` object at `Priv+0x7D0`, with a 32-bit value slot at object `+0x50` (`Priv+0x820`). This is constructor/type evidence. No selected runtime `Priv` instance, value-read event, or generation receipt joins the object to a consumer.

The first missing edge is the selected instance's same-owner/generation relation to a read or serialization path. The first BSI/DDP sink remains `UNBOUND_NOT_TRACED`; do not borrow Line DRC `dynrng` or bitrate-sink evidence for this field.