# GTC local v3 approval-status erratum

The Wash-verified local v3 owner-availability amendment was approved by the user on 2026-10-30. Its adoption package and LOCAL-CONTRACT identify the status as `USER_APPROVED_LOCAL_CONTRACT`; approved v2 remains unchanged.

The hash-bound `ProgrammeState-v3.schema.json` still contains proposal-era annotations (`$id` ending in `v3-local-proposal`, title saying proposed, and `x-proposal-status=PROPOSAL_PENDING_USER_APPROVAL`). Those annotations are stale and do not revoke or override the user's approval recorded by the adoption package. Do not edit the sealed v3 payload in place; this erratum supplies the current adoption status.

The approved availability shape is: in METERING, meter is INITIALIZED from actual canonical state and the other eleven owners are UNINITIALIZED with no generation, hash, or state reference; measurement receipt is null. A successful MEASUREMENT parent swap retains terminal meter bytes, installs the receipt and eleven actual cold owner states, and transitions to READY atomically with one phase-generation increment.

This resolves the v2 status-schema representation mismatch. It does not provide the eleven runtime constructors, execute production root materialization, authorize placeholder hashes/state references, or prove READY swap/whole-chain execution. `OutputOwner` still requires the approved plan-bound provider identity contract.
