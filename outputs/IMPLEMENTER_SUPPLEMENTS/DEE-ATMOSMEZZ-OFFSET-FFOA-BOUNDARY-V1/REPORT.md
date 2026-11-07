# AtmosMezz input offset and FFOA boundary

The input XML schema declares offset and ffoa as optional strings with schema default auto. The checked XSD is not bound to the executing DEE image, and the text values in a source XML receipt do not establish selected runtime values.

In the selected static host-normalizer branch, offset auto reads the existing metadata-map offset. FFOA auto uses metadata FFOA minus metadata offset plus normalized offset. The normalizer writes the results to a host-local property list and an AtmosMezz field at +0x130. The XML leaves are not joined to those policy fields, and the host list is not joined to converter PrivContext +0xa8.

The selected converter consumer reads offset and first_frame_of_action from PrivContext +0xa8. Offset goes through std::stod, so literal auto without a numeric prefix is rejected. The XML key ffoa is not joined to first_frame_of_action. Effective values, owner/generation, Reader seek sample, units, and frame meaning remain UNBOUND or NOT_CAPTURED. This package does not combine input offset/ffoa with filter start/end or time_base.
