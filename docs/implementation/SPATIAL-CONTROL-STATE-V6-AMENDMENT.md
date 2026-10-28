# Spatial-control persistent state v6

This local amendment replaces the implementation's v5 byte layout. It does not rename or alter the washed `selected-adm-installed48-state.v3` source package and does not claim native byte-for-byte parity. Field behavior follows the selected-profile F03W/F05R56 specifications and the verified Wash field-projection and prefix-diffusion supplements. Any serialization choices below are JOCForge local schema decisions.

## Single owner and staged after-state

`SpatialControlState` is the single spatial-control committed root. It now contains the F03W render/control fields and the complete F05R56 `ReaderTargetState` as one nested substate. The parent callback stages both in the same `SpatialControlAfterProposal`; neither leaf commits locally.

F05H's `previous_active` transition is stored once in the nested F05R56 state. The F03W `previous_marker_active` name is a read-only projection of that value, not a second advancing field. Current F05H marker integers remain a separate callback result in F03W's `markers_i32` field.

The installed48 producer's after-state is consumed into the spatial-control proposal. Its source cursor, physical PCM rows, current48 rows and 37 regular-tail diffusion states are projected into the v6 root; the temporary producer state is not retained as a second committed copy. Prefix outputs select modes and lanes from those 37 regular-tail states. They do not own eleven additional filter instances.

The physical control snapshot is a typed host-ordered array: `Bed6([u32; 6])` or `Object13([u32; 13])`. Bed words 6 through 12 are undefined by the approved selected bed branch and are omitted. They are not zero-filled or described as recovered values.

## Canonical byte encoding

The preimage starts with `JOCSCS6\0`. All U32/U64 and F32 bit words are little-endian. Fixed arrays are row-major and do not carry a count. Optional values use a U8 tag (`0` absent, `1` present); present variable values carry a U32LE element or byte count. The order is:

1. Callback index, source cursor, normal callback count, remaining cached drains, and retired flag.
2. Optional last-normal physical 38×1536 PCM words; optional host-ordered control rows. Each control row has a U8 word-count tag (`6` for Bed6, `13` for Object13) followed by exactly that many U32 words. Then the cached phase byte.
3. The validated F01E event/control canonical bytes.
4. Regular raw history 37×1536, previous physical PCM 38×1536, and previous-current rows 48×1536.
5. Diffusion-tail count 37, then one presence tag per physical-order regular tail. A present tail carries a U32LE length and `DTAIL1\0` bytes: length-prefixed `DSTATE1\0` filter-state bytes, an output-presence tag, and four length-prefixed previous-output lanes as exact F32 words.
6. Feature source vectors 48×24 and source scalars 48.
7. Current and previous target XYZ, IDs, attributes, and active flags. `previous_marker_active` is represented only inside the nested reader state.
8. A U32LE length plus the complete canonical `ReaderTargetState` bytes.
9. Assignment I32[15], marker I32[15], matrix F32[15][48], writer-set U32LE count, then its ordered unique U8 target slots.

The v6 SHA-256 is computed over these complete bytes. Callback-local packets, scratch, provider payloads, pointers and unknown bed fields are excluded. The selected profile dimensions remain immutable admission metadata and contribute no bytes.

## Limits

The current producer connection constructs a pending normal-callback after-root and nested F05R56 state. It does not admit the receipt or commit the root. Cached-drain staging, the enclosing multi-owner `CallbackTransaction`, full-programme replay, output generation, OpenJOC comparison and product acceptance remain separate work.
