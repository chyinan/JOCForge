# Git release ref policy

The public clean-release candidate retains exactly one Git ref:

| Ref | Purpose | Required target |
| --- | --- | --- |
| `refs/heads/master` | sole clean implementation branch | includes expected baseline `6a70c04b51c74bd050be61dc90a3ae83f30e006a` |

No tags, remotes, notes, stash ref, replace refs, alternates, grafts, or
application snapshot refs are part of the public release namespace. Any
application-managed snapshot is external workspace state and must be absent
from the release object database. Reflogs are not retained as provenance
evidence.

The reproducible audit is `scripts/audit-git-release.ps1`. It checks the
allowlisted refs, complete object reachability, unreachable/dangling objects,
packed and loose object inventory, reflogs, alternates, replace refs, grafts,
and strict clean-side identifiers across the retained history. The release
gate requires zero unexpected refs, zero unreachable objects, no alternates or
replace refs, and a reachable history scan with no forbidden administrative
identifiers.
