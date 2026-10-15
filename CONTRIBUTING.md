# Contributing to JOCForge

JOCForge is an independent implementation of public ADM, BW64, E-AC-3, and
JOC requirements. Contributions must stay within that public provenance
boundary.

## Clean-room expectations

Migration gate: `IMPLEMENTATION_READY=NO`. Algorithm implementation is paused
until the human approves the architecture and supplies the approved specification
package, including its GTC owner projection. During this migration, that package
is the only authorized behavioral source. The general provenance guidance below
does not authorize choosing additional algorithm references or using historical
code, Git history, audit exports, research tools, vectors or outputs. A fresh
implementation agent must receive only the isolated baseline and approved package.

- Use public standards, public documentation, and legitimately redistributable
  open-source references.
- Do not submit proprietary encoder code, disassembly, private symbol/table
  dumps, copyrighted source media, or material copied from restricted work.
- Explain the public source or specification basis for non-obvious behavior.
- Keep external validators and decoders in the documented validation boundary;
  do not copy their serialization or decoder internals into JOCForge.

## Before opening a change

Run the same checks used by CI:

```powershell
rtk cargo fmt --all -- --check
rtk cargo check --workspace --locked
rtk cargo clippy --workspace --all-targets --locked -- -D warnings
rtk cargo test --workspace --locked
rtk git diff --check
```

Tests should cover changed behavior, malformed input, and any public profile or
carriage contract affected by the change. Do not include generated audio,
private audit output, or local validator binaries in a change.

## License for contributions

Contributions must be original work or based on publicly available
specifications and legitimately redistributable references. By intentionally
submitting a contribution for inclusion in JOCForge, you submit it under the
Apache License, Version 2.0, unless you clearly state otherwise. No separate
CLA or DCO sign-off is required for this first release.
