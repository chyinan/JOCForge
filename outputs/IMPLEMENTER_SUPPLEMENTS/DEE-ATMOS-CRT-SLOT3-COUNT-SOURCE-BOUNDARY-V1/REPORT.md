# DEE ATMOS CRT SLOT3 COUNT SOURCE BOUNDARY V1

Wash verified the source package identity and recorded the following bounded finding.

- scope: One slot3 allocation initializer count source and its static CRT failure edge.
- count_source: No explicit function parameters; U32 loaded from a component-local global.
- normalization: Global value 0 becomes default 512; nonzero values below 3 clamp to 3; values >=3 are preserved.
- allocator: Normalized count is passed as element count with element width 8; retry after null uses count 3.
- n_join: No ADM physical N parameter is present on this call edge; global's upstream writer/owner remains untraced, so global-to-N relationship is UNBOUND.
- failure_edge: Double allocation null returns unsigned -1, propagated through CRT error table to PROCESS_ATTACH FALSE; runtime allocation failure NOT_RUN.

Open status: `SLOT3_COUNT_FROM_COMPONENT_GLOBAL; ADM_PHYSICAL_N_JOIN_UNBOUND; RUNTIME_NOT_RUN`. Runtime: `NOT_RUN`.

- Global upstream writer/owner/generation not traced.
- Static possible CRT failure route only; attempt-12 error 1114 source remains NOT_IDENTIFIED.
- No DEE run or allocation-failure observation.
