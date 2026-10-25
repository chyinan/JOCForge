# F06ND to selected F12 BSI control-field binding

This supplement binds the clean F06ND output fields to the corresponding selected F12 BSI/per-block DRC fields. It is a clean-spec composition contract, not a native binary identity claim.

## DRC control source mapping

The source owner is the successful F06ND scheduler output for the same `CodecSourceIndex`/source generation consumed by F12. The selected F06ND success contract returns `RF_packed_word_u16`, `RF_flag_u16`, six `line_packed_words_u16[i]`, and six `line_flags_u16[i]`; RF and all six line flags are 1 on this success path.

| Clean source field | F12 clean BSI/per-block field | Presence | Width / conversion | Bitstream position |
|---|---|---|---|---|
| `RF_flag_u16` | `compre` | 1 on successful selected F06ND output | 1 bit | fixed header-relative bit 50 |
| `RF_packed_word_u16` | `compr` | written when `compre=1` | U16 high byte: `word >> 8`, then 8 bits | fixed header-relative bit 51 when present |
| `line_flags_u16[i]` | per-block `line DRC presence` for block `i` | 1 on successful selected F06ND output | 1 bit | after the block's optional short and dither fields |
| `line_packed_words_u16[i]` | per-block `line DRC code` for block `i` | written when its presence bit is 1 | U16 high byte: `word >> 8`, then 8 bits | immediately after that block's line-presence bit |

The fixed E-AC-3 prefix occupies 45 bits before `dialnorm`; `dialnorm` is a same-programme selected-loudness U5 at bit 45. `compre` follows the 5-bit dialnorm at bit 50. The optional 8-bit RF code follows at bit 51. Mixing/information/additional-BSI fields follow and conditionally move later positions.

Line DRC is not contiguous with the RF code. F12 writes each block in block order 0 through 5; inside a block it writes optional five-channel short flags, optional five-channel dither flags, then the line-presence bit and optional 8-bit line code. Its absolute frame bit offset is derived from the current bit cursor and preceding optional syntax, not a fixed six-byte region.

The U16-to-U8 operation is a bit projection, not endian byte slicing or numeric rounding. F12 writes the resulting code MSB-first as an 8-bit field.

## Fields not written to these BSI locations

The selected F12 BSI/per-block DRC input has one RF compression code and six line dynamic-range codes. It has no selected mapping for `RF_aux_packed_word_u16`, `RF_limiter_packed_word_u16`, `line_aux_packed_words_u16`, or `line_limiter_packed_words_u16`; do not serialize those audit/limiter projections into the RF or line fields. Other BSI values such as mix/information data and JOC additional-BSI payload retain their separate profile/F13 sources.

## Source-stage and commit boundary

F06ND and F12 both consume the same source generation. The BSI plan is built from that frame's F06ND outputs and fixed selected-profile metadata. F12 trials read the same staged BSI; only the successful candidate commits its staged codec state. This supplement does not grant the BSI projection a separate state owner or claim the global GTC/source commit is complete. The approved F06WH source-stage composition remains the parent transaction boundary.
