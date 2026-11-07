# Line-mode DRC to dynrng: first typed break

The two previously sealed ADM comparisons are reused without rerunning A/B. In both windows, `music_standard` changes the observed per-block `dynrng` sequence relative to `film_light`; `music_light` matches `film_light` in the measured fields and window. The equality is sample- and window-specific and does not establish a general alias.

The static Line path conditionally maps the XML strings to plugin profile-tree U32 values (`music_standard` → 3; `music_light` → 2) when the token reaches the selected plugin and the inspected mode-1 path is active. `FUN_1801535b0` writes the matched tree-node U32 to snapshot `+0xA0`. This is not the core curve selector: the separately traced DRC core takes an I32 custom-profile flag and U8 preset selector (or four U8 custom selectors), then feeds a selected curve pointer and per-block F32 state into a U16 candidate/reference code producer. The XML string-to-selector edge and numeric mode-to-Line/RF identity are `UNBOUND`.

The first typed break is at the plugin's per-frame pack path. The same snapshot pointer reaches `FUN_1801764e0`, but the bounded validator and packer do not directly read or copy `snapshot+0xA0`. In the inspected zero branch, the first row helper `FUN_180182850` receives a selected row pointer and local vectors, not the `+0xA0` scalar or its address; that row's owner/alias relationship to `+0xA0` is `UNBOUND`. Its next nested conversion, `FUN_1801431f0`, is only established as a generic string-like conversion, not a codec or BSI record. No typed edge reaches the U8 curve selector or per-block dynrng producer.

Thus the available evidence does not support a mode-to-curve, coefficient, or per-frame dynrng rule. The conditional DDP_COMPR values in the static profile map are a separate field and must not be substituted for the core U8 selector or per-block `dynrng` code. The music-light comparisons remain observations, not a profile equivalence rule.

This continuation only reuses sealed Wash evidence and checks its manifests. It does not run Ghidra, DEE, an A/B, or a PCM comparison; it does not inspect payloads or alter implementation files.
