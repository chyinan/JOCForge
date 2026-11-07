# DEE ADM PHYSICAL N READER DESCRIPTOR BOUNDARY V1

Wash verified the source package identity and recorded the following bounded finding.

- scope: One selected inner/outer Reader construction path and its immediate descriptor count use.
- inner_reader: FUN_180185160 constructs outer Reader then FUN_1801852e0 creates inner AtmosReaderPrv from CompositeMaster; inner +0x58 initialized NULL.
- descriptor_consumer: Later FUN_1801852e0 reads inner +0x58, then U16 descriptor +0x0A; paired CompositeMaster U32 +0x30 is passed to a helper whose result is stored in outer param_1[3].
- distinct_count_domain: K candidate = individual_record_count + sum(group_entry_count), accumulated U32 and low U16 written to K +0x0A; typed join to inner Reader descriptor is UNBOUND.
- physical_n: Input-only examples N=36 and N=38 do not join to Reader descriptor count.
- first_break: Nonzero producer of inner Reader +0x58 descriptor pointer remains UNBOUND.

Open status: `PHYSICAL_CHNA_N_TO_READER_DESCRIPTOR_UNBOUND; K_TO_READER_DESCRIPTOR_UNBOUND; RUNTIME_NOT_RUN`. Runtime: `NOT_RUN`.

- One selected construction path only; no other writers searched.
- No global support/absence claim.
- DEE runtime and PCM remain NOT_RUN/NOT_READ.
