# F06NR approved source leaf materialization

Status: `VERBATIM_APPROVED_SOURCE_LEAVES`.

This package copies the sixteen c7 IMPLEMENTER_PACKAGE leaves cited by the F06NR source-frame input-only crosswalk into the isolated implementation workspace. Every file is byte-for-byte identical to its approved package identity. This is a source-availability aid only; it adds no algorithm and does not change the F06NR interface or its NOT_RUN numerical after-state status.

The companion package `F06NR-SOURCE-FRAME-INPUT-ONLY-CROSSWALK-V1` supplies the typed input, clock, state-shape, carrier-zero-slot, and generation-rejection crosswalk. It does not contain nonzero expected DSP results. This source bundle also contains existing block-level model vectors for reference; they must not be promoted to full-source expected after-states or native evidence.
