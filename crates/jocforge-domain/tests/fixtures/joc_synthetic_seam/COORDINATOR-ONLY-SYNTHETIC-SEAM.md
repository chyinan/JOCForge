# F05JL coordinator-only synthetic seam

## Decision

The seam is allowed only for input-only coordinator tests. It accepts
caller-provided synthetic receipts for receiver_qmf, temporal, geometry,
fit180, profile_retry and huffman_id14. Each receipt must carry the actual
canonical input/output material from a frozen vector, not a digest-only claim.

This does not relax the production no-skip rule. The seam neither executes nor
claims to execute F04C, F05JC, fit180, retry or Huffman producers. It validates
evidence that the synthetic fixture says they executed. It is forbidden as a
production F05JL entry point and cannot create PASS receipts for missing
stages.

## Canonical receipt contract

Stage order is receiver_qmf, temporal, geometry, fit180, profile_retry,
huffman_id14. Every stage receipt requires:

- generation equal to dispatch;
- status PASS and executed true;
- owner_token exactly JOC_DISPATCH_{dispatch}:{stage};
- provider file/selector, provider input/output SHA-256 and complete canonical
  input/output objects;
- input and output shape equal to the provider shape;
- input/output encoding and complete material;
- SHA-256 of sorted compact UTF-8 JSON material for JSON_CANONICAL_V1;
- SHA-256 of decoded bytes for HEX_BYTES;
- provider hashes recomputed from their canonical objects;
- first upstream_output_sha256 null and every later input binding the preceding
  stage output SHA-256.

The huffman output material is the complete ID14 body hex. It must equal the
top-level id14_body_hex, its decoded length must equal id14_body_bytes, its bit
length must equal id14_body_bits and unused terminal padding bits must satisfy
the frozen Huffman provider receipt.

The provider and exact shape table are in synthetic-seam-provider.json.
Provider identity is SHA-256 over sorted compact UTF-8 JSON.

## Clock and lineage

For dispatch d: source=d-1, callback zero-based=d+1, callback one-based=d+2,
writer=d-1 and AU is null for d<3, otherwise d-3. All six generations and owner
tokens bind d. The top-level clock must match those equations.

A missing, reordered, failed, non-executed, wrong-generation, wrong-owner,
wrong-shape, wrong-material-hash, broken-upstream or body-mismatch receipt is a
hard validation failure. No downstream stage is synthesized or inferred.

## Allowed output and state boundary

Success returns an immutable synthetic validation receipt and the verified
ID14 body proposal. The receipt lists all six input/output hashes, clock
identity, body length/hash and source vector identity.

The seam commits no state. commit_count is zero and
joc_state_delta_committed=false. It must not modify or propose receiver/QMF,
temporal, geometry, fit, retry, sequence, writer, FIFO or root generation
state. F04C/F05JC/joc root/global CallbackTransaction remain outside this
slice. Production code must run the six numerical producers and use the normal
transaction contract.

## Failure semantics

Validation failure returns no body proposal and no validation receipt. There
is no partial stage execution, state change or retirement side effect because
this seam has no persistent owner. failure_stage names the first invalid
synthetic receipt. The enclosing test may assert fail-stop separately, but this
seam cannot issue a production RETIRED receipt.

## Evidence boundary

The frozen source vector is pinned by path, byte count and SHA-256 in the
generated vectors. This supplement is clean-room test-interface design.
It does not promote the joc root, F04C, F05JC, programme restart or global
CallbackTransaction.
