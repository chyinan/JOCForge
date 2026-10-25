# F13JO model complexity to selected F12 additional-BSI field

## Clean model producer and wire consumer

The approved F13JO model parses a validated EMDF containing ID14 and derives `joc_complexity = U8(joc_object_count + 1)`. For the selected model profile, `joc_object_count=15`, so the model value is 16. The model's validated ancillary payload carries that value to `encode_frame_with_bsi`.

The selected F12 writer places the additional-BSI fields MSB first: presence at bit 90; length code at bits 91–96; payload identifier at bits 97–104; and the complexity U8 at bits 105–112. The cursor after the field is bit 113. The model parser labels it `complexity_index_type_a`; the F12 wire contract defines it as the eight-bit complexity field.

`carrier_block` is a separate model input used to select an EMDF carrier block (selected value 2). It is not the complexity U8 and is not derived from `object_count`.

## Source limits

The selected native BSI field is statically sourced from a validated configuration-count byte. Existing native evidence does not join that byte to a semantic ID14 reader or `object_count+1` producer; it also shows the field can exist when ID14 body/queue/serializer objects are absent. The selected numeric value 16 matching the model formula is not a producer, owner, or generation join.

The clean model field link is closed for implementation from the approved F13JO/F12 contracts. Native ID14-to-BSI source parity remains `UNBOUND`; a complete valid OAMD+ID14 wrapper run is `NOT_RUN`. This package does not claim whole F13C readiness or native parity.
