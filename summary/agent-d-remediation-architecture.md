# Agent-D remediation architecture

Verified 2026-09-02 from the clean reconstituted `master` tree.

## Critical boundary

The current workspace was recreated from a bundle containing only the approved
`refs/heads/master` history. The former workspace, including its object store
and untracked audit/evidence material, is kept at
Historical workspace locations are outside the implementation and validation
paths and are not referenced by releasable clean documentation.

## Finding-to-module map

- F-004 crosses `reduce_scene_with_previous`, `assign_sources`, and
  `continuity_cost`. Previous assignments are currently stored in current
  source iteration order but looked up by the numeric source identity, which
  fails when LFE filtering leaves gaps. The clean fix is an identity-keyed
  pure lookup plus a sparse-index regression test.
- F-003 crosses `parse_adm_objects` and the encode admission boundary. The
  parser currently selects the first block and retains timing strings without
  applying or rejecting unsupported semantics. The narrow clean policy should
  accept only a full-programme static block shape and reject extra blocks,
  non-zero block runtime, and non-full object timing before PCM encoding.
- F-002 is primarily an evidence-boundary issue. The closure script currently
  writes `unintended_leakage=0` from source membership labels and only records
  render process exit status. The clean fix must compute thresholded metrics
  from independent source/matrix reconstruction and public profile speaker
  projections, then make the acceptance fields fail closed.

## Supporting minor cleanup

- Remove the unused legacy authored-object admission API that contradicts the
  active N-to-15 reducer, or align its public contract with the reducer.
- Name and document the non-normative numeric controls in analysis, reduction,
  writer, and file scanning; tests should exercise their boundary behavior.
- Use one authoritative closure runner/status matrix and state what remains
  structural evidence versus open perceptual research.
