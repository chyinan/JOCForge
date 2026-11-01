# F12 multi-ADM DialNorm/API boundary

## Cross-input observations

Three authorized ADM BWF inputs were measured under the same 5.1 R138-equivalent reference profile: 48 kHz, 1024 kbps, 1770-4 metering, dialogue intelligence off, threshold 100, DRC none, and `file_position` window 2160–2208. DEE succeeded for all three. The native log did not state the measured scalar's unit. A bounded custom helper read seven bytes from each temporary output's first independent syncframe and found U5 values 21, 14, and 17; it read zero payload bytes and retained no raw prefix. The official OpenJOC CLI cross-check was not run.

Those three U5 observations match the already-washed selected-LUFS model **if** the DEE scalar is `selected_lufs`. This is useful cross-input empirical support for that conditional mapping under this profile. It does not establish the scalar's native unit, a DEE scalar-to-frame/API join, or the serialized BSI rule. A fourth First Love measurement reported `-21.010569` with no stated unit; no U5 observation was made for it.

## Static API trace

The Ghidra trace follows the aggregate binary32 scalar through owner state `+0xF6C` and encode configuration `+0x114`, then a bit-preserving copy into `EmpireEnc` state `+0x3C`. No unit conversion or rounding was found before the encoder API boundary. This does not establish the final BSI field.

The sibling `dee_ddp_encoder.exe` trace independently follows frame metadata `param_3+0x60` through a `[1,31]` clamp to `ddpi_enc_setparams` ID `0x12`. It does not join this field to the measured scalar. Optional IDs `0x30`/`0x31`, the DPLC setdata float path, and the `abs(x)+0.5` logger path are also not joined to the measured scalar or final BSI. The actual BSI writer/bit packer remains untraced. The prior offset erratum remains in force: the `int*` expression `param_1+0x3C` in `FUN_180178DC0` addresses byte `+0xF0`, not the scalar at byte `+0x3C`.

## Disposition

Reference-only supplement; `implementation_ready=false`. It records conditional observations and the API boundary, not a new product algorithm rule. Keep native unit, scalar-to-frame/API identity and generation, and final BSI serialization `UNBOUND` / `NOT_PROVEN`. The next proposed observation is metadata-only and remains `NOT_RUN`; this package does not authorize an encode or output read.

DEE processed the authorized ADM inputs internally for measurement. No PCM was decoded or manually read. No output payload beyond the seven-byte header probe was read; no raw prefix was retained. No private key, HMAC, or license content was read, copied, or hashed. The third ADM file was streamed for SHA-256 and byte-identical alias creation as disclosed by the incoming receipt; no PCM decoding occurred.
