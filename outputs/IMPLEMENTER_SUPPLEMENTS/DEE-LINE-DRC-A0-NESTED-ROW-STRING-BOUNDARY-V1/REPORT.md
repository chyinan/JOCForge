# Line-mode DRC nested row-string boundary

## Bound static behavior

From the zero branch of the per-frame row packer, the selected-row pointer is passed to a generic string-like consumer. That helper reads source length at +0x10 and capacity at +0x18; capacity determines inline versus heap storage. With start=0 and a finite source length, the UINT64_MAX request sentinel clamps to the available source length, so the helper copies the full string and writes a NUL terminator.

## First break

This is a generic string copy, not a typed codec/BSI record. The selected row has no owner join to the line-mode snapshot+0xA0 U32, and no codec/BSI owner is identified. The next byte-copy helper body is not inspected. Runtime string length/capacity/bytes, generation, clock, and output effect remain unobserved.

The upstream manifest SHA-256 is 45292eba4506d5c084e47cd538c903f2542e9931e08741ffd9c4f22ba275c53e; package-seal file SHA-256 is 534e15c1d22d37b54d2314dda146615904b854b4b46e5a9d42450322b5b1f878. Wash verified all seven payload hashes and the SHA256SUMS seal. The upstream agent reports normal and python -O checks passing. Raw trace material is excluded.
