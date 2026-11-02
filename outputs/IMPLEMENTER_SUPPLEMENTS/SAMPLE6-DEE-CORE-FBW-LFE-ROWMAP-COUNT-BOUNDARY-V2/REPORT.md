# Sample 6 DEE core FBW/LFE row-map count boundary — producer update

Recipient: implementation evidence supplement  
Disposition: table-derived internal counts are bounded for selectors 0–7; their ADM source and live selector range remain unbound.

## Newly bound count producer

Ghidra 12.2 static evidence identifies `FUN_1403479c0` as a writer of `context+0x19130` and `context+0x19132` on the main `FUN_140343530` path. It copies S16 `context+0x2c` to normalized layout selector `context+0x1912c` and S16 `context+0x30` to LFE-present state `context+0x1912e`, then computes:

```text
base_count = S16[0x1405f4ba0 + 2*layout_selector]
context+0x19130 = base_count
context+0x19132 = base_count + (S16(context+0x1912e) != 0 ? 1 : 0)
```

Both fields are two-byte signed stores. For table selectors 0–7, `base_count` values are `[2,1,2,3,3,4,4,5]`; the corresponding six-wide map rows have valid descriptor prefixes `[3,2,3,4,4,5,5,6]`. Thus the slot count is table-derived in `1..5`, and the layout-position count is table-derived in `1..6` for those selectors. This is not an explicit clamp.

For selectors 0–7, those counts stop at the corresponding valid row prefix before trailing `0xffff` sentinels. The writer does not check selector range locally, and no sentinel guard precedes the mask write. The exact source of the live selector and LFE-present values is not bound; therefore selector behavior outside 0–7 remains unbound. A second writer `FUN_140348000` has the same table formula, but its caller is not bound in this package.

## Relation to arbitrary ADM track count

`context+0x19130` and `context+0x19132` are internal row-map/layout dimensions, not the ADM physical row count. This package does not connect either source field `context+0x2c/+0x30` to `fmt.nChannels`, CHNA rows, bed rows, object tracks, or the DDP frame-view count. Do not route 36 or 38 directly into these fields.

No runtime value was captured. The smallest future observation is `FUN_1403479c0` entry at `0x140343556`: capture the selected context and the two source S16 fields, then verify the four normalized/stored values. No DEE run, PCM/payload/key read, or raw7 capture occurred.

## Upstream seal

- Package: `row_count_field_producer_static_wash_v1`
- MANIFEST.json SHA-256: `569a1c870494b4697c3343daed4e6fcdc4c2b81e120273d76c93e1c076db24cb`
- package_seal.json SHA-256: `544259910c756bc2e48c32b05388ba67ad36602abf9c20448773c8a30dd3b5b6`
- Four payloads and SHA256SUMS independently matched; no upstream checker script was provided.
