# Event-grouping boundary erratum

Wash correction; this package supersedes prior parent-grouping language for the F13C short-window and serializer/refill/writer observations.

The reported marker is in a function-epilogue/security-check neighborhood, not a parent-call entry, access-unit identity boundary, or source-generation boundary. Reclassify the F13C window pairs as SAME_PARENT_UNBOUND / SAME_THREAD_SEQUENTIAL_COHORT_ONLY. Reclassify serializer/refill/writer triplets as sequential-cohort triplets only.

Length and pointer-value equalities and the bounded writer-argument no-alias result remain value observations within sequential cohorts. Serializer-to-refill causal flow, same-parent, same-AU, and source-generation identity remain UNBOUND.

No production rule follows. Keep the historical packages immutable and apply this erratum whenever they are consumed.
