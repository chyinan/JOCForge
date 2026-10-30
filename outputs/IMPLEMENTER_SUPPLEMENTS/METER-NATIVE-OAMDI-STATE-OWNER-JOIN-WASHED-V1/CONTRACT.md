# Native per-object OAMDI state-owner join

## Bounded run-025 finding

Run 025 logs the table-B projected-record token `R`, bed/object updater native index 0, target-object token `O`, and per-object state-owner token `S`. The updater's projected record and state owner are `R` and `S`; the state store receives the same `S`, the same `O`, and native index 0, copying a 30-byte record. This binds the updater-to-state-store owner identity within run 025. The same effective EC-3 job completed successfully; only the output hash is recorded.

Table-B getter and bed/object updater still use distinct index expressions: absolute loop counter versus loop counter minus a native start/base. Both equal zero for this first observed record; do not equate their domains or join either to F06OF target slot 0. Run-024's formatter gate was zero, so its conditional first-F32-to-compact-field branch did not execute; this evidence does not show that the table-B scalar changed stored state or EC-3 output.

OAMDI-to-EVO serializer functions did not fire in this selected EC-3 job. The per-object state to OAMD/BSI serializer or queue join remains unobserved. F06OF target/control word and generation joins remain `UNBOUND`.
