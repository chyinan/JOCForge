# RF DRC F98 vector entry producer boundary

The vector header destination is bound to the owner field at +0xf98. The generic copy receives one 8-byte source entry from the caller-local span [RSP+0x28, RSP+0x30), whose slot is written from RSI. Ghidra models the generic element as longlong, but the semantic owner and type remain UNBOUND.

The bounded same-caller continuation has no later exact +0xf98 reference. The source entry is not joined to the RF StringType field, the separate +0xf88 allocation pointer, or numeric RF state. The first consumer and RF DRC effect remain UNBOUND; no neighboring-offset inference is used.
