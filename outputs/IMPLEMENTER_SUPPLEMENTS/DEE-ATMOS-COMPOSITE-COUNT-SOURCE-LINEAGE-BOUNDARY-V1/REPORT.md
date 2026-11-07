# CompositeFormatCount source boundary

In the inspected composite-building path, CompositeFormatCount comes from a per-source shape metadata view or a transform of shape-map entries. The count can be read from that view, or derived from entries marked as channel-bearing and narrowed to U16. The chosen layout branch was not captured dynamically.

The per-source record vector has its own element count. The ADM physical N path is separately bound from the input descriptor's U16 row count through open configuration into a layout object, with a static 1..128 guard. No typed pointer, owner field, or explicit conversion joins that physical-N value to the per-source metadata views or source-record vector count in the inspected path. The checked producer is not a direct load/copy from the ADM N field; ultimate derivation from the same ADM rows remains UNBOUND.

The shape-derived count later reaches the Reader/frame consumer, but its runtime value and branch were not captured. This slice separates count domains; it does not claim a global absence or an end-to-end supported range. No DEE run, PCM, payload, key, or license access occurred.
