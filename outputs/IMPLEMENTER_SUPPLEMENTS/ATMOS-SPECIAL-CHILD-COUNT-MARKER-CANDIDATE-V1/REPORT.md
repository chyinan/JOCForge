# Atmos special child-count marker candidate

Ghidra 12.2 static evidence binds the native special-count rule to tagged
six-byte channel-map records. For each child, count records whose byte at
offset `+2` is `0xFF`; take the maximum count across children; then add 10.
The tag producer writes `0xFF` for each opened object-source row. The source
row field at `+0x4C` supplies the map slot, while the record's `+4` field is a
separate ordinal. The deeper semantic identity of `+0x4C` is not established.

The seven washed ADM metadata vectors support a practical candidate: bed-row
vector lengths sum to 10, and CHNA Object-track counts are 26 for six samples
and 28 for `人是猫`, yielding conditional single-child candidates 36 and 38.
Each candidate matches the sample's declared CHNA track count. This is useful
for a best-effort implementation trial, but it assumes one marker per CHNA
Object track and that the marked records fall in one child. The metadata does
not establish either runtime mapping or the per-child partition.

Do not substitute AXML object-node counts: each is one greater because it
includes the bed object, and the native predicate is the generated tag byte.
The selected AtmosReader path statically selects special mode through a
generated key-2 flag, but the audited run receipts do not capture the runtime
mode byte or child U16. Preserve the native formula and keep those runtime
values unbound. This package contains no source/decompilation, audio payload,
PCM, private key, or license data.
