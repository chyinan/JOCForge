# DEE ATMOS CRT SLOT3 ATTACH FAILURE PATH BOUNDARY V1

Wash verified the source package identity and recorded the following bounded finding.

- scope: One component B initializer table slot and its direct CRT error-table propagation path.
- slot: Initializer table slot index 3, one-based ordinal 4, in a seven-qword table including sentinel.
- failure_path: Requested allocation null triggers retry with count 3; retry null returns unsigned -1; error-table walker treats nonzero initializer return as failure; PROCESS_ATTACH returns FALSE if the table result is nonzero.
- runtime: Allocation failure NOT_RUN; this slot as attempt-12 1114 source NOT_OBSERVED; specific failing initializer NOT_IDENTIFIED.

Open status: `SLOT3_STATIC_ALLOCATION_FAILURE_TO_CRT_FALSE_PATH_BOUND; RUNTIME_FAILURE_AND_ATTEMPT12_CAUSE_UNOBSERVED`. Runtime: `NOT_RUN`.

- One slot only; no other initializer table entries scanned.
- This does not attribute runtime error 1114.
- No DEE attempt or runtime allocation observation.
