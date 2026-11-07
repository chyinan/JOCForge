# DEE preferred-downmix XML setter boundary — V2

This V2 supersedes V1 only for the static constructor/first-writer path. It retains the same XML token map and open runtime/audio boundaries.

## Bound static behavior

The Atmos DDP property preferred_downmix_mode is registered as an XML string with exact tokens loro, ltrt, ltrt-pl2, and not_indicated; the parser maps them to 32-bit codes 0, 1, 2, and 3. The static default candidate is loro. Effective behavior for omitted or empty XML remains untested.

In the inspected constructor path, neither MixMode constructor directly initializes MixMode+0x50 before parsing. The parser passes its mapped code to the typed setter, which calls a helper and stores the helper's return at +0x50. The optional callback was initialized to null, so that helper skips validation and returns the pre-existing +0x50 value. The setter therefore writes that existing value back; the existing value's initializer is not joined to the XML ordinal.

## First reader and remaining gap

A typed getter reads +0x50 and a formatter reverse-maps it to a token. No selected processing-stage callsite joins the getter/formatter to an audio or BSI consumer. Runtime value and owner/generation remain unobserved. The V1 package was not modified; this V2 records the revised sealed source package and the same bounded status.

The upstream manifest SHA-256 is d3495410ffacd0c942ba04731e87fc4ae8ba76046d9f83cb700ed10395bd9b49; its seven-payload/eight-checksum verifier passed normally and under python -O. The installed DLL was not rehashed. No raw Listing, source code, XML sample, media, PCM, payload, key, or license is included.
