# Preferred downmix selected encode-pass read boundary

The existing typed getter receipt binds `FUN_180171f50` as a reader of `MixMode+0x50`; its only direct caller is the token formatter `FUN_180171ce0`. The formatter maps the code back to a token string, but its exact function-entry xref count has no direct code caller.

A bounded instruction/decompiler check of selected encode-pass `FUN_1801475f0` found no direct `Priv+0x980` access (`MixMode+0x50`). This does not prove that no other consumer exists.

The stored MixMode value's path into a selected audio/BSI consumer and any preferred-downmix effect remain `UNBOUND`; runtime value remains `NOT_RUN`. No speaker/AU parity or Lo/Ro-vs-Lt/Rt output claim is made.

Source package `DEE-ATMOS-DDP-PREFERRED-DOWNMIX-SELECTED-PASS-READ-BOUNDARY-WASH-V1` manifest SHA-256 `a6e1aa6c1f2cd4cce6fa80fc9cc8ee408b29f0a2ddeccd8a91f08ff782392e78`, source checksum-list SHA-256 `19f84ef9fe79468a37a1152d63103e82d29ea5ea6bda14e1ff3fd1ee4820d59a`; its normal and `python -O` checker passed. Ghidra 12.2 read-only/no-analysis; no DEE/XML run or media/key/license access.
