# Public data_rate to typed codec-rate consumer boundary

## Disposition

PLUGIN_SELECTOR_TO_TYPED_KBPS_BOUND; PUBLIC_XML_STRING_TO_SELECTOR_UNBOUND; SETTER_OWNER_JOIN_UNBOUND; RUNTIME_NOT_RUN.

The static plugin path maps a signed I32 selector to typed U32 kbps and reaches the first typed rate validator. The public XML strings are not yet connected to those selector codes, so this does not close the public XML option end to end.

## Bound plugin path

FUN_180178dc0 accepts selector codes 0 through 5 and writes the accepted value to encoder-open config word 10 at byte offset +0x28. FUN_1801788a0 passes the same config block through FUN_1801a7d50 to FUN_1801ab520. At the examined call, param_3 is the config pointer and *(int *)(param_3 + 5) reads the same word 10 at byte offset +0x28.

FUN_1801ab520 maps selectors to a typed U32 kbps value: 0→384, 1→448, 2→576, 3→640, 4→768, 5→1024. FUN_1801fe3d0 is the first typed rate validator on this path. It reads typed key 3 (U32 kbps) and key 4, then calls FUN_180200570 with the constraint min_kbps <= rate_kbps - rate_offset and rate_kbps <= max_kbps. The active registry row, key-4 semantics, and rate_offset unit are not bound.

FUN_1801ff1e0 also reaches typed setter endpoint FUN_1801c1470, which stores a type-3 U32 to handle +0x18370. The owner/generation join between this endpoint and the open-request config remains conditional/unbound.

## Limits and verification

Wash recomputed the source manifest, seven listed files, MANIFEST.sha256, eight parent manifests, and current host/plugin binary hashes. Both binaries matched their listed hashes. The source package has no formal package-seal file. Its CHECK.py reports normal and python -O success; Wash did not execute it.

No DEE encoding, runtime observation, media/payload/key/license access, or implementation contact occurred. This supplement does not infer XML value mapping by enum order or numeric equality.
