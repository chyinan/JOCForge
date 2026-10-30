# Current-job spatial evidence: per-run boundaries

Source identities are the current prepared job XML SHA 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9, normalized config SHA 02c68b5dffddc2557cc6b841268a4d4a0395377c5643607ff1f72b8dd44c24a0, and authorized 38-track source SHA 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1. The updated observation reports two distinct native runs under those same job/config/input identities.

Run v4 contains a same-parent, same-thread matrix-writer-to-consumer call: selected-index and descriptor pointers match across the call, and the consumer receives a 6x48 matrix. The attached 1152-byte F32LE matrix payload comes from v4 only.

Run v5 captures 15 packed masks and row records, stopping at the first matrix writer. Across v4/v5, active flags, selected ordinals and the first six geometry records compare equal. Runtime generations are different. Therefore no causal packed-mask-to-v4-matrix join is established; do not combine the runs into a single same-generation proof.

The selected config has six selected rows, from a 15-row capacity and 48 descriptor columns, while source WAVE has 38 physical tracks. These are distinct domains. Host-frame, ADM sample-position, fit-generation and U04 invocation joins remain NOT_CAPTURED. U04 numeric output, Type10/six-plane, model replay and Gate1 remain open. The matrix is a DEE reference payload, not model output or product acceptance.

Updated research manifest SHA-256 167dba14ba433843fcd2d9e10ffd77d08565aa7d1e9eb8374b26dc7a4ef73b8e. Earlier interpretations that implied a same-generation packed-mask-to-matrix chain are superseded.
