# Composite selector: per-item handler-selection boundary

## Finding

The sealed Ghidra 12.2 Wash package binds a per-item source-path suffix classifier to the storage-handler dispatch. The classifier is case-insensitive. Its observed map includes `wav → 3`, `damf/atmos → 2`, `rpl → 1`, `xml → 7`, and `mxf → 4`.

The ADM/BWF storage registration installs a WAV master-open wrapper in slot 3. **If** this J/K item's nested source path ends in `.wav`, that wrapper obtains a provider, calls the provider's open method, stores the provider on J after success, and forwards J's `+0x08` K output slot to the open method. The actual nested path suffix is not present in the receipts, so the selected handler for this J/K remains `UNBOUND`.

## K-count implication

The existing candidate ADM/BWF count rule is `low16(I + ΣG)`, where `I` is the individual-record count, `G` is each group's entry count, accumulation is U32, and the stored K count is U16. This package does not reach the selected handler's K-count writer for the opened J+`0x08` slot. Applicability of that formula to this exact J/K instance therefore remains `UNBOUND`; do not infer the nested source suffix from the top-level ADM/BWF filename.

## Disposition and limits

This is a scoped conditional route and first-break receipt, not a global claim about other storage-handler slots. It adds no selected suffix, runtime K count, generation join, 36/38 result, DEE execution, or audio output. It does not authorize an implementation change that assumes this route is selected.

Wash verified the upstream manifest SHA-256 `82ec118eb0bc4bba8133bdcaadacf12c33f4c3b3fd86adcef6cb5419f179c516` and reran its evidence checker normally and under `python -O`. The upstream directory contains no separate `package_seal.json`; this supplement records that fact rather than inventing a seal.
