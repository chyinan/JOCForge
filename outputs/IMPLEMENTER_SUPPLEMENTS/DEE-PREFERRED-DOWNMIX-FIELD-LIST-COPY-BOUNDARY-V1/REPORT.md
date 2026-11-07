# Preferred downmix field-list copy boundary

## Bound path

The selected builder path calls `FUN_1801a2580` with the field-pointer vector from `dee::proc::Priv+0x210`. That function initializes a separate `dee::ParametersContainer` output object at `Priv+0xec0` and passes the vector to `FUN_1801a23e0`. The helper counts qword elements, allocates storage, and copies the pointer-vector bytes; neither function dereferences the elements.

The vptr write in `FUN_1801a2580` belongs to the separate `ParametersContainer` at `Priv+0xec0`; it does not type or construct the `Priv+0x210` member. No `+0x210` member constructor/vptr writer, preferred-downmix ordinal join, or first audio/BSI consumer is established by this path.

## Boundary

This receipt only identifies a vector-copy route and the separate output object's type. The member element owner/type, ordinal mapping, runtime generation and audio effect remain `UNBOUND` / `NOT_RUN`. No global absence claim is made.

## Provenance

Source package `DEE-ATMOS-DDP-PREFERRED-DOWNMIX-FIELD-LIST-CONSUMER-BOUNDARY-WASH-V1` has MANIFEST SHA-256 `68b826e4383c1b71ee4023a8045ad8eefa5d7f61825eb5064a9fc0882de4f29a`; its `SHA256SUMS.txt` SHA-256 is `13e560ed3a004fb335a9d066ba30b10dbac14ee08a945fbd51af190e4eea3c39`. The source verifier passed normally and under `python -O`. Ghidra 12.2, read-only/no-analysis; no DEE run or media/key/license access.
