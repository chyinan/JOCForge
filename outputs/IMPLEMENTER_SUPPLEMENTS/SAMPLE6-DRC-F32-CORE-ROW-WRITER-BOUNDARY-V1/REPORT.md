# Sample 6 core F32 row writer boundary

Recipient: implementation evidence supplement  
Disposition: static boundary only; no runtime values or algorithm change authorized.

## Verified result

The upstream Wash-only package was independently checked against its sealed manifest. `CHECK.py` passed in normal Python and with `python -O`.

For the matched DEE image `dee_ddp_encoder.exe` (SHA-256 `0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532`), the checked static path binds a per-channel state pointer at `channel_instance+0xdf8`, its aligned initializer, the 30-word row initialization, and the typed reader. The row is `[state+0x23e34,state+0x23eac)`: 120 bytes interpreted as six blocks of five F32 values, with 0x14-byte block stride.

The initializer writes 30 zero U32 words at open time. The checked lifecycle and three bulk-clear candidates do not write or clear this row. In the reviewed in-image scope, the only row store found is that initializer; no generation-time typed writer/copy was found. Runtime row values therefore remain `UNBOUND`. The constructor zeros must not be used as runtime values.

The next static boundary is the unresolved host/API consumer of data pointer slot `0x140610538`, which points to `FUN_140350d00`. The package does not establish whether that external dispatch path writes the row before processing.

## Scope and limits

This is Ghidra 12.2 static evidence plus a symbolic input-only vector. It is bounded to the hash-bound DEE image and listed functions/range checks; it does not prove a whole-process absence of writers. There was no new DEE run, runtime capture, PCM/syncframe/payload access, or private-key/license access. It establishes no PCM difference cause and supplies no replacement F32 values or algorithm rule.

## Upstream seal

- Package: `sample6-matched-drc-f32-core-row-writer-boundary-wash-v1`
- MANIFEST.json SHA-256: `e33a3254d5877b6cce046cb39447e30c47dc88807663e5c716548e1d4e3665f9`
- package_seal.json SHA-256: `fe57a3e738250a3e406ded705e8cd141aa261d14d9e51d118466c2d782b3b6c1`
- MANIFEST.sha256 SHA-256: `e8b88f641eec7f92e815c84003aea0a0bfba8eda2a518bd1150a8214a9f1d370`
- REPORT.md SHA-256: `8d72aa50b29d30eb7d41133e9a57cc15e47653c2ccda30f7e6ed6ab687bb68b5`
- EVIDENCE.json SHA-256: `b5b065c9c394c12c2b694756d603605b6f882897ed0f121cc3960e077384f005`
- INPUT-ONLY-VECTOR.json SHA-256: `dfc14251f872d3dd026281fff4674dcd174733646ae929739bede9899ced16dd`
- SOURCE-IDENTITIES.json SHA-256: `552827309465962a1d147763e86ef7bccd34ff566d09ed03753dee8088bca8a7`
- CHECK.py SHA-256: `da2c5b9090b2041b95532ef4404ead1227345c490e81dabcc1772337eddb7c27`
