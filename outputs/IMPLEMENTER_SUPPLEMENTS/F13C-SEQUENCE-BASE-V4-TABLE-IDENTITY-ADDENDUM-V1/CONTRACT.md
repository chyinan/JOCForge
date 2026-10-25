# Sequence-base V4 table identity addendum

Status: `FAMILY_B_TO_V4_TYPE_TABLE_STATICALLY_BOUND; SELECTED_INVOCATION_AND_RUNTIME_GENERATION_UNBOUND`.

The reviewed `Family B` wrapper path and the V4 sequence-base source package refer to the same exact encoder image and the same profile-type dispatch table, Type 10 row, Type 11 row, and nested callback-method table. The table entries for Types 10 and 11 share that nested method table. The method table contains the V4 sequence-base handler and parent-processor entries. This closes the static table identity for Family B; it does not identify a selected runtime invocation.

Within Family B, the event-row handler and subsequent parent-row processor receive the same profile-row owner expression. V4's static contract describes the conditional Type 10/11 sequence-base event behavior on its profile row. Together, these establish a same-row static path for the mapped table family, not a same-source-generation or actual selected-run receipt.

The nearby tag-218 parse/serialize candidate from the earlier review is not part of this V4 type table: its function-pointer table is identified as a generic MPEG2VideoDescriptor class. Likewise, the same numeric value 218 in unrelated helper status paths does not establish event semantics.

The following remain `UNBOUND`: actual selected type/selector; selected event occurrence; host source/job invocation of Family B; same-row runtime generation across constructor callback and later parent call. Family A's relation to the V4 table is also not established. This addendum supersedes the earlier Wash status that left Family B's table-to-V4 mapping open; it does not close the whole F13C chain or authorize an algorithm change.
