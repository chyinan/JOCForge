# Serializer/refill/writer boundary observation

Native observation only; not an implementation specification. In one bounded run, 64 serializer/refill pairs reportedly had equal lengths and the same output/body pointer. Only 63 complete writer triplets were available after preserving process-boundary truncation.

Seven writer ABI argument locations were checked at those 63 entries; none directly aliased the refill body pointer. This excludes only direct aliasing at those locations. A copy, transform, wrapper, or object-field relation remains possible. No ADM source-generation or AU join was captured.

Implementation effect: none. Infer no frame-layout or writer-input rule. The smallest follow-up is a targeted trace of the writer argument object or immediately preceding ordinary call boundary.
