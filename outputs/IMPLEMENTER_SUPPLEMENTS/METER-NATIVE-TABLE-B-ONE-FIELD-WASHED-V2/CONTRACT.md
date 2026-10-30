# Native table-B one-field chain

## Correction to earlier wording

The sealed v3 research package supersedes the prior Wash wording that described a separate map key/value pair or left the node field's key-versus-value role unresolved. Static allocator/comparator/ordinal-getter/writer evidence identifies an ordered tree set of signed 32-bit elements: the same element is compared for ordering and returned by ordinal access. There is no separate key/value pair in this container.

## Bounded dynamic field chain

In run 022 callback 1, ordered-set element at local ordinal 0 with signed value 3 is read, normalized by the native converter to 3, written as a four-byte scalar to native table-B row 0 at byte offset 20, and consumed as row 0 by the same table owner. Only that source scalar was read; no PCM or table-row payload was read. This binds one native set-element-to-table-field-to-consumer chain.

The source semantic code name and its connection to F06OF remain unbound. F06OF's model adapter maps its own target slot `s` to control52 row `s` and control16 row `1+s`; no typed adapter joins the native set element/table-B field to that target slot or word. The observed native value 3 equals one F06OF model word value 3, but numeric equality is not identity. Native callback ordinal is not F06OF generation `g+1`.

Run-019 and run-020 row indices 0 and 1 remain distinct observations. Table count 16, 52-byte stride, row position, and scalar equality are not cross-system identity evidence. The global candidate-ID/source-weight join also remains outside this table-B finding.
