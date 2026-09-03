# Git release ref policy

The public clean-release candidate retains exactly one Git ref:

| Ref | Purpose | Required target |
| --- | --- | --- |
| `refs/heads/master` | sole clean implementation branch | contains the preserved clean parent `1f4c44bfd1a508f299e1aa7fd3c825b759103151` and the frozen release-candidate history |

No tags, remotes, notes, stash ref, replace refs, alternates, grafts, or
application snapshot refs are part of the public release namespace. Any
application-managed snapshot is external workspace state and must be absent
from the release object database. Reflogs are not retained as provenance
evidence.

Codex-local working refs under `refs/codex/turn-diffs/*` belong to the
development workspace namespace. The host may recreate them while a task is
active; their presence is not evidence that the project release namespace is
unclean. A release export therefore uses an explicit bundle of
`refs/heads/master` and a fresh object database, without copying development
refs, remotes, stash, reflogs, or unreachable objects. Agent-D4 audits the
resulting frozen candidate repository rather than the instrumented development
workspace.

The reproducible audit is `scripts/audit-git-release.ps1`. It checks the
allowlisted refs, complete object reachability, unreachable/dangling objects,
packed and loose object inventory, reflogs, alternates, replace refs, grafts,
and strict clean-side identifiers across the retained history. The release
gate requires zero unexpected refs, zero unreachable objects, no alternates or
replace refs, and a reachable history scan with no forbidden administrative
identifiers.
