# D7 audit execution model

> Clean-room C8 handoff; this document defines audit isolation and does not adjudicate provenance or a public release gate.

## Locations

D7 uses three separate locations supplied by the auditor:

1. **Frozen candidate** — `<candidate-root>` is the read-only provenance authority. No test, build, archive, audit report, generated evidence, or temporary file may be written below this path after the candidate fingerprint is recorded.
2. **D7 audit workspace** — `<audit-root>` stores fingerprints, reports, logs, and generated audit artifacts.
3. **Disposable execution copy** — `<execution-root>` is an independent non-local clone used for cargo, scripts, tests, and any tool that may write build or evidence output.

The audit workspace and execution copy must be outside the frozen candidate. Do not use object alternates, hardlinks, shared worktrees, or a local clone operation that couples their object stores.

## Required sequence

1. Verify the candidate starts at the expected attached `refs/heads/master` commit and record the candidate fingerprint in the audit workspace.
2. Create the execution copy with an explicit non-local clone, for example:

   ```powershell
   git clone --no-local --single-branch --branch master $env:JOCFORGE_CANDIDATE_ROOT $env:JOCFORGE_EXECUTION_ROOT
   ```

3. Run engineering, cargo, validation, and audit commands only from the execution copy. Set `CARGO_TARGET_DIR` to a directory under the audit workspace if a command must directly read another path; never let cargo target output resolve into the frozen candidate.
4. If `git archive` is required, use the absolute path in `$env:JOCFORGE_ARCHIVE_PATH` outside the candidate. Never use a relative archive path while the candidate is the current directory.
5. Reproduce the candidate fingerprint at the end from the frozen candidate. A changed status, tree hash, file manifest, object count, pack hash, ref, reflog, alternate, replace ref, or other candidate state is an audit failure requiring a fresh candidate.

## Candidate immutability fingerprint

Record, where applicable, the following in the audit workspace before and after execution:

- `HEAD` object ID and symbolic HEAD;
- all refs, with only `refs/heads/master` retained;
- `git status --short --branch`;
- reachable, unreachable, dangling, and unexpected object counts;
- object database and pack hashes;
- HEAD tree hash;
- candidate file manifest and SHA-256 values;
- reflog, alternates, replace-ref, graft, and worktree state.

The fingerprint is reproduced by the D7 auditor independently. Files created in the audit workspace or execution copy are not candidate mutations and must never be used to alter the frozen candidate.

The auditor must resolve and validate every writable location before any command that can write:

```powershell
$candidate = (Resolve-Path -LiteralPath $env:JOCFORGE_CANDIDATE_ROOT).Path
$audit = (Resolve-Path -LiteralPath $env:JOCFORGE_AUDIT_ROOT).Path
$execution = [IO.Path]::GetFullPath($env:JOCFORGE_EXECUTION_ROOT)
$archive = [IO.Path]::GetFullPath($env:JOCFORGE_ARCHIVE_PATH)
$cargoTarget = [IO.Path]::GetFullPath($env:JOCFORGE_CARGO_TARGET)
foreach ($writable in @($audit, $execution, $archive, $cargoTarget)) {
    if ($writable.StartsWith($candidate + '\', [StringComparison]::OrdinalIgnoreCase)) { throw "writable D7 path is inside frozen candidate: $writable" }
}
git -C $candidate rev-parse HEAD
git -C $candidate symbolic-ref --short HEAD
git -C $candidate show-ref
git -C $candidate status --short --branch
git -C $candidate count-objects -v
git -C $candidate ls-tree -r --full-tree HEAD
Get-ChildItem -LiteralPath $candidate -File -Recurse | Get-FileHash -Algorithm SHA256
```

Save the command outputs as the start fingerprint, repeat the same commands at the end, and compare the normalized results outside the candidate. The candidate, audit workspace, execution copy, archive, and `CARGO_TARGET_DIR` must all remain disjoint from the frozen candidate.

## Prohibited actions after freeze

Do not run `cargo build`, `cargo test`, `cargo fmt`, validation scripts, `git archive` with a relative output, or any other mutating command from inside the frozen candidate. Do not repair, delete, or clean a file in the candidate after the start fingerprint. If an output path is unclear, stop and resolve it to an absolute external path first.
