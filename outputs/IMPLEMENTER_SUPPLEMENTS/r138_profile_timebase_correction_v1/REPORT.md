# R138 profile time-base correction

## Correction

Wash verified that the hash-bound R138 execution XML (`fa65143f0bd7de52b36367137e61d681baa4c17824aded10be26317e7d1018a4`) specifies `time_base=embedded_timecode`. The earlier `file_position` value in `HOST-ENCODE-ROUTE-PROFILE-FACTORY-SELECTION-FOLLOWUP-V1` is retracted for R138 and must not be reused.

This addendum corrects only that profile field. R138 remains route `encode_to_atmos_ddp`, input type `audio.adm`; the static route-to-profile option lookup remains closed. Profile-owner-to-selected-factory-index remains `UNBOUND`, the actual host dispatch callsite remains unidentified, and the task-generation token remains unobserved.

R2's checkout does not contain the original R138 XML file, so the corrected field is attributed to Wash's hash-bound source verification. No native run was started.
