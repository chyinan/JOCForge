# DEE time_base XML-value writer boundary

Reference-only Wash addendum. It refines the time_base range-consumer supplement but leaves product support open. `IMPLEMENTATION_READY=NO`.

The configured StageRunner key selects `FilterDDPJocEncPlugin`. Its `time_base` schema is `xs:string`, values `file_position` and `embedded_timecode`, with a constructor default of `file_position`. The selected DDP `Priv+0x550` StringType is statically connected to the same-Priv `FUN_18014fd90` comparison/range consumer.

The host's accepted XML value object has not been connected to a typed writer/callsite for the selected DDP instance's `Priv+0x550` current value. The observed host literal belongs to a template/default builder; constructor initialization proves only a default. Runtime value is `NOT_CAPTURED`, the value writer is `UNBOUND`, and actual range/seek is `NOT_RUN`. The prior time_base supplement remains useful for the consumer formula; this addendum narrows only the missing setter edge.