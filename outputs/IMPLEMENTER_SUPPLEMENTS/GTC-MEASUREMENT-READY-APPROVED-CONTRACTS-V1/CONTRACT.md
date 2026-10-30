# Approved GTC measurement and programme-state contracts

This bundle copies four exact files from approved IMPLEMENTER_PACKAGE manifest c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a. It defines selected-profile ProgrammeState schema, owner roots, MeasurementTransaction, receipt fields, terminal meter finalization, single parent METERING to READY swap, fail-stop and restart semantics. Meter EOF uses its own Q normal callbacks and three fresh-zero drains; main cached drains are separate.

The schema defines twelve owners and receipt shapes, but schema availability is not proof that production constructors, real warm state, parent admission or a READY transition executed. Use each owner contract for actual state bytes and receipts; do not populate from fixtures or treat a proposal as committed. Real full prepass, terminal hash, receipt, READY and output remain execution evidence to be produced.
