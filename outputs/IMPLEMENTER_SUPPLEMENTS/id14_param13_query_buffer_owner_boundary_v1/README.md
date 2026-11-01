# ID14 query-buffer root owner boundary

Ghidra 12.2 static tracing places the ID14 count input root inside a plugin-created query buffer. The callee's param3 index 1 is resolved to a local scratch root within that buffer, not to the caller's input pointer. The buffer is sized and cleared before the query builder runs.

The selected count constructor reads two U32 cells at root offsets +0xc170 and +0xc174, subtracts them, validates the result, and stores the resulting count for ID14 U6 serialization. The cells are also used in loop/dimension arithmetic and a positivity gate, but their semantic names, units, and upstream record source remain UNBOUND. A whole-buffer zero fill is only a candidate initializer: exact field coverage, cursor/capacity relation, and later indirect writes were not established.

Parameter-13 remains a separate property-list path. Static co-residency in the worker does not prove a common parent generation. No native run was performed because the available R138 profile does not demonstrate keyless execution.
