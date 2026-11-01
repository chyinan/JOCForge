# Source converter runtime identity clarification

This Wash-only clarification keeps the R139 static copy separate from the R128 runtime component identity. Both are assigned the semantic role `ADM_INPUT_CONVERTER`, while their SHA-256 values differ. The difference alone is not treated as proof of incompatible versions.

The exact R128 runtime converter image was statically analyzed and contains the FFOA-auto source-retention branch and the file-position first-frame rule. R138 records the same authorized input, executable, and DDP Atmos processing image, but does not capture the converter module hash. Keep the R128 converter rule provisional for R138 until that module identity is directly bound.

The R128 storage-support image was also inspected with a focused metadata-key search. The inspected functions did not resolve the source virtual writer for the two time fields. The first absolute sample-position consumer in spatial/fit remains unobserved. No native run or model execution was performed.

Verify with:

```text
python verify_source_converter_runtime_identity.py
python -O verify_source_converter_runtime_identity.py
```
