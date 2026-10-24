# Selected policy-parameter provider

This F09PA direct slice loads only the hash-pinned selected policy parameter artifact and exposes its 19 scalars as exact F32 bit words or I32 values. The artifact is checked by exact byte length and SHA-256 before JSON parsing. The provider admits only profile `eac3-joc-48k-1024k-acmod7-uncoupled-nonspx-policy-v1` with the exact profile fields in the washed parameter record.

The asset order is D06-T-NORMAL, D06-W, D06-N, D07-P, D07-A, D07-T, D07-H. Shapes and types are respectively F32[1], I32[4], F32[1], F32[5], F32[4], F32[3], and F32[1]. The provider preserves F32 words rather than converting or computing with them. It serializes these 19 values in asset/element order as I32/F32 little-endian and reproduces the washed 76-byte digest.

Washed error mapping is preserved: absent provider data returns `PARAMETER_UNAVAILABLE`; a non-selected profile returns `UNSUPPORTED_PROFILE`; artifact identity, syntax, type, or shape failures return `POLICY_PARAMETER_ERROR`. This provider is stateless and has no lifecycle or transaction effects. The two supplied profile-rejection recipes assert no state advance.

The vectors and result file cover parameter identity and serialized values, two profile rejection cases, and 17 frozen-model consumer comparisons. The previous local review additionally records the missing/profile/corrupt error partition and parameter-field validation. This implementation does not implement refresh, gain/db selection, normalization, or lifecycle consumers; those algorithmic functions remain outside this provider-only slice. Whole F09 and product readiness remain `NO`.

Source package `MANIFEST.json` SHA-256: `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`.

Parameter artifact SHA-256: `a99d077900058eb4a862f5ea29bc87408bf6d7a4f4cc42e3da66ae6c68a93c6a`.
