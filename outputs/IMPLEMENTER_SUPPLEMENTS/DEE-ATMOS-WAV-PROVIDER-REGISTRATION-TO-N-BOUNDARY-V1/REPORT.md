# DEE ATMOS WAV PROVIDER REGISTRATION TO N BOUNDARY V1

Wash verified the source package identity and recorded the following bounded finding.

- scope: FUN_1803ecb80 registration arguments and two direct registry-table stores; callback bodies not inspected.
- provider_id: 3
- primary_callback: FUN_1803ec660 stored at registry+0x20 via setter FUN_1804cb200.
- secondary_callback: FUN_1803ec480 stored at registry+0xE0 via setter FUN_1804cb1d0.
- physical_n: No N, row count or layout owner is present in these registration arguments; provider handle to ADM layout N owner (+0x28) remains UNBOUND.

Open status: `ADM_BWF_PROVIDER_CALLBACK_REGISTRATION_BOUND; CALLBACK_BODY_AND_PHYSICAL_N_JOIN_UNBOUND; RUNTIME_NOT_RUN`. Runtime: `NOT_RUN`.

- Callbacks themselves were not inspected.
- No DEE execution or runtime capture.
- No N owner, input generation or product-readiness inference.
