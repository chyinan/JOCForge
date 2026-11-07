# LFE temporary-buffer dispatch boundary

## Finding

The upstream Wash receipt follows the gathered 256-F32 pointer into `FUN_140340640`. Before its first helper call, that function performs no pointee read or write. It compares the pointer with its 16-byte aligned-up address, then passes the **unadjusted** pointer as both RCX and R8, with EDX=`0x100`, to one of two helpers: `FUN_1403174c0` on the equal branch or `FUN_140317530` otherwise.

The meaning/unit of EDX and the read/write extent of either helper were not established. This is the next concrete boundary in the LFE audio path.

## Continuity and limits

The dispatcher later passes the restored pointer to `FUN_14036e350`; pointer identity across the inspected call path is statically bound. Buffer-content continuity after either helper remains `UNBOUND`. No helper body, other caller, runtime branch, sample owner/generation, numeric parity, or correlation cause is established here. DEE was not run and no PCM, payload, key, or license was read.

The upstream package manifest SHA-256 is `5a5854ae6ac7576879046b3bb04aa667512944eaf57498d97f152f939b588658`, and its package-seal SHA-256 is `174cb7f872b8233094990af167ab6325392409e19c3d764c7fa5739cd428a4a4`. Both normal and `python -O` checkers passed. The raw Listing JSON is excluded from this supplement.
