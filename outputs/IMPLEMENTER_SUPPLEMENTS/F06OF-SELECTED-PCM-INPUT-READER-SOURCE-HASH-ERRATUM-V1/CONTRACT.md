# F06OF reader source-hash erratum

Status: `APPROVED_MODEL_SOURCE_REFERENCE_MISMATCH; READER_MODEL_CROSSWALK_UNBOUND`.

The earlier washed supplement `F06OF-SELECTED-PCM-INPUT-READER-STATIC-CONTRACT-V1` names `specs/06-oar-frontend-input-and-clock-join.md` with SHA-256 `2a98742b9032b900bb42da7f6a7c404a835c9539fbb42fd1a13a1a1d8f83c4ef`. That digest does not match the current approved package manifest or the bytes of that file. The approved manifest SHA is `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`; its entry and the actual file hash are `2a98742b9032b900bb42da7f6a7c404a835c9539fbb55040939d7d025c8ce423`.

Do not use the mismatched digest as provenance for the current approved F06OF model. The static reader findings were separately checked against the converter PE image and remain static evidence, but their crosswalk to this approved model file is unverified until the differing source is identified. This correction does not establish the selected-run identity, physical-row-to-measurement-descriptor join, or EOF/drain behavior. Keep those relationships `UNBOUND`; `IMPLEMENTATION_READY=NO` for the complete selected reader chain.
