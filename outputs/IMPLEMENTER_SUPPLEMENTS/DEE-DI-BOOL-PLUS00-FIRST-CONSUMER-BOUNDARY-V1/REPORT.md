# DI Bool normalized config first-consumer boundary

The setup Bool at +0x94 is bound to normalized configuration byte +0x00. The configuration pointer is forwarded through the selected adapter chain, but the first downstream constructor reads configuration byte +0x14; the separate Wash receipt shows +0x14 is initialized to constant 1. That consumer neither reads nor retains the Bool-derived +0x00 byte.

The Bool-to-meter-result dependency and runtime branch selection remain UNBOUND. This is a bounded constructor-chain result only; it does not establish product-wide absence or an audio effect.
