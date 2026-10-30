# OAMD coordinate grid provider and parser dependency

Status: approved clean parameter source identified; independently recomputed integer grid matches all 79 values. This is a model/provider result, not a new native runtime or complete ADM validation.

## Production provider

Approved package asset: `parameters/oamd-coordinate-grid-q15.json`, schema `oamd-coordinate-grid-q15.v1`, UTF-8 raw file length 1169 bytes, SHA-256 `b1814fec24ac57a45c5bf2eacaba91c0c581df19362c6ca7da4cc911ed336243`. `parameters/adm-installed48-providers-v1.json` binds that exact raw file and `parameters/reader56-target-profile-v1.json` identifies it as the coordinate grid provider with XY count63 and Z count16. F05O.6 in `specs/05-oamd-row-and-timing-wire.md` defines the generation and decoder rules. These are ordinary nonsecret functional values, not test-only fixture data or a native provider requirement.

For integer j use exact nonnegative integer arithmetic, with a wide enough intermediate:

* `xy[j] = min(32767, (32768*j + 31) div 62)` for `0 <= j < 63`.
* `z_magnitude[j] = min(32767, (32768*j + 7) div 15)` for `0 <= j < 16`.

Here div is integer floor; no floating-point conversion or generic round-to-even is involved. Both arrays contain nonnegative Q15 integer values 0..32767. These values can be represented exactly in `[u32;63]` and `[u32;16]`: unsigned integer widening changes neither values nor indices. For identity checks, the asset SHA covers the exact JSON file; it is not the binary array SHA. Recomputed canonical U16LE packed hashes are XY `08fd929d47ee561920225e56828e08e8246e0b4a8c8a547e6bfdce6574965534`, Z `5a948eec6707dcdf3833fbecdee6c28fcb5c5614ca99a05a07f974fa7eaf1460`. Separate U32LE packed hashes of the widened values are XY `9e0d88ece1babc8cec8e53911dd52830ca08e9f00cdfcfde8bce7f5bf69b9a6d`, Z `bc119b5b79f29e36b492f74f7b4ceb900a9cf126b3801ed1d7befabfcaeef963`; they identify this explicitly defined packing, not the raw JSON or native bytes.

Useful boundary facts from the generated values: XY indices 0,31,62 produce 0,16384,32767; Z indices 0,15 produce 0,32767. Both grids are strictly increasing. No zero-filled default, separately recovered compiled table or undocumented native identity is needed.

## Decode use

For each unmuted object, read XY as unsigned six-bit codes and use `xy[min(code,62)]`, so code63 maps to index62. Read Z as sign bit plus unsigned four-bit magnitude 0..15; take the magnitude grid value and apply the sign in a signed type after widening. A negative zero becomes numerical zero; do not negate an unsigned word. The table provides decoded coordinates, not the original Q15 input: encoding and reconstruction are distinct. Muted objects use the F05O.6 stated defaults and do not consume object-coordinate fields. Follow F05O.6 for component/gain/extension and other row semantics; F05O/F05T specify section boundaries and trim, not just row-count extraction.

## Can complexity avoid coordinate tables?

F13JO requires a complete validated current EMDF, semantic parsing of ID11/OAMD and ID14 and 16 actual OAMD slots; a 16-row header or an ID14 count alone is not equivalent to that validation. Complexity itself is `U8(parsed_ID14_object_count+1)` and does not numerically use coordinate magnitudes. A deliberately scoped validator could, in principle, consume and validate every relevant row field without materializing decoded coordinate values, but the inspected approved contract does not authorize replacing semantic validation with a header-only shortcut or give a separately validated coordinate-free implementation. Do not claim the latter exists. For a full F05O.6 decode, use the approved grid above. If a future coordinate-free validation is proposed, prove grammar, field constraints and slot accounting independently and have Wash review its precise equivalence; no internal token alone is a reason to hold the entire audio product path.

## Verification and boundaries

The checker computed both arrays from the index formulas before opening the expected asset and compared all 79 entries, schema, exact raw-file SHA, ranges, strict ordering and boundaries. It reproduced the two approved U16LE hashes and separately computed the documented U32LE hashes. Regular and optimized Python both passed with explicit runtime checks. Existing `test-vectors/OAMD-COORDINATE-GRID-VERIFICATION.json` records a prior 79-word comparison to research arrays; that record is not a new native capture. Full protected OAMD+ID14 parser, audio pipeline, native runtime and ADM playback were NOT_RUN for this delivery.
