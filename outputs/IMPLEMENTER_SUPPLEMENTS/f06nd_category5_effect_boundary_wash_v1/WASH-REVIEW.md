# Wash review: F06ND category-5 selected-process boundary

The selected AtmosProcessor process path receives frames/channels, copies samples into an internal buffer, and calls the Atmos backend. No category-5 parameter, InputControlEvent vector, or category-5 byte-record branch is present on that selected process interface. The approved profile's four zero logical records remain its selected-profile state; a byte layout is not applicable at this process boundary.

A separate generic storage-event API accepts InputControlEvent vectors, but no source/owner/generation join connects the selected category-5 submission to it or to a DRC/BSI consumer. That broader effect remains UNBOUND.

This is a partial negative boundary from static evidence only, not proof of global absence. Do not invent a category-5 record layout for the selected process path. No native run or media payload was accessed.
